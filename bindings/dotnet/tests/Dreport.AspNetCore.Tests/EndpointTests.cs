using System.Net;
using System.Net.Http.Json;
using System.Text;
using System.Text.Json;
using Dreport.AspNetCore;
using Dreport.Service;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.Mvc.Testing;
using Microsoft.AspNetCore.TestHost;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Xunit;

namespace Dreport.AspNetCore.Tests;

/// <summary>
/// Spins up an in-memory ASP.NET Core host for each test using TestServer
/// directly, so we don't need a Program.cs entry point. Verifies the
/// stock /api endpoints behave the same as the original Axum backend.
/// </summary>
public class EndpointTests
{
    private const string Template = """
        {
          "id": "aspnet-test",
          "name": "AspNet Test",
          "page": { "width": 210, "height": 297 },
          "fonts": ["Noto Sans"],
          "root": {
            "id": "root",
            "type": "container",
            "position": { "type": "flow" },
            "size": { "width": { "type": "auto" }, "height": { "type": "auto" } },
            "direction": "column",
            "gap": 5,
            "padding": { "top": 15, "right": 15, "bottom": 15, "left": 15 },
            "align": "stretch",
            "justify": "start",
            "style": {},
            "children": [
              {
                "id": "title",
                "type": "static_text",
                "position": { "type": "flow" },
                "size": { "width": { "type": "auto" }, "height": { "type": "auto" } },
                "style": { "fontSize": 14, "fontWeight": "bold" },
                "content": "Hello"
              }
            ]
          }
        }
        """;

    private static HttpClient Build(string prefix = "/api")
    {
        var builder = new WebHostBuilder()
            .ConfigureServices(s => s.AddRouting().AddDreport())
            .Configure(app =>
            {
                app.UseRouting();
                app.UseEndpoints(e => e.MapDreportEndpoints(prefix));
            });
        var server = new TestServer(builder);
        return server.CreateClient();
    }

    [Fact]
    public async Task Health_Returns_Ok()
    {
        var client = Build();
        var resp = await client.GetAsync("/api/health");
        Assert.Equal(HttpStatusCode.OK, resp.StatusCode);
        var json = await resp.Content.ReadFromJsonAsync<JsonElement>();
        Assert.Equal("ok", json.GetProperty("status").GetString());
    }

    [Fact]
    public async Task Render_ReturnsPdf()
    {
        var client = Build();
        var payload = new { template = JsonDocument.Parse(Template).RootElement, data = new { } };
        var resp = await client.PostAsJsonAsync("/api/render", payload);
        Assert.Equal(HttpStatusCode.OK, resp.StatusCode);
        Assert.Equal("application/pdf", resp.Content.Headers.ContentType?.MediaType);
        var bytes = await resp.Content.ReadAsByteArrayAsync();
        Assert.True(bytes.Length > 100);
        Assert.Equal((byte)'%', bytes[0]);
    }

    [Fact]
    public async Task Render_InvalidTemplate_Returns400()
    {
        var client = Build();
        var payload = new { template = "not a template", data = new { } };
        var resp = await client.PostAsJsonAsync("/api/render", payload);
        Assert.Equal(HttpStatusCode.BadRequest, resp.StatusCode);
    }

    [Fact]
    public async Task Layout_ReturnsJson()
    {
        var client = Build();
        var payload = new { template = JsonDocument.Parse(Template).RootElement, data = new { } };
        var resp = await client.PostAsJsonAsync("/api/layout", payload);
        Assert.Equal(HttpStatusCode.OK, resp.StatusCode);
        var json = await resp.Content.ReadFromJsonAsync<JsonElement>();
        Assert.True(json.TryGetProperty("pages", out var pages));
        Assert.Equal(JsonValueKind.Array, pages.ValueKind);
    }

    [Fact]
    public async Task ListFonts_IncludesNotoSans()
    {
        var client = Build();
        var resp = await client.GetAsync("/api/fonts");
        Assert.Equal(HttpStatusCode.OK, resp.StatusCode);
        var families = await resp.Content.ReadFromJsonAsync<JsonElement[]>();
        Assert.NotNull(families);
        Assert.Contains(families!, f => f.GetProperty("family").GetString()!.ToLowerInvariant().Contains("noto"));
    }

    [Fact]
    public async Task GetFontBytes_KnownVariant_ReturnsTtf()
    {
        var client = Build();
        var resp = await client.GetAsync("/api/fonts/Noto%20Sans/400/false");
        Assert.Equal(HttpStatusCode.OK, resp.StatusCode);
        Assert.Equal("font/ttf", resp.Content.Headers.ContentType?.MediaType);
        var bytes = await resp.Content.ReadAsByteArrayAsync();
        Assert.True(bytes.Length > 1000);
    }

    [Fact]
    public async Task GetFontBytes_Unknown_Returns404()
    {
        var client = Build();
        var resp = await client.GetAsync("/api/fonts/DoesNotExist/400/false");
        Assert.Equal(HttpStatusCode.NotFound, resp.StatusCode);
    }

    [Fact]
    public async Task CustomPrefix_RemapsAllEndpoints()
    {
        var client = Build("/dreport/api");
        var resp = await client.GetAsync("/dreport/api/health");
        Assert.Equal(HttpStatusCode.OK, resp.StatusCode);
        var oldRoute = await client.GetAsync("/api/health");
        Assert.Equal(HttpStatusCode.NotFound, oldRoute.StatusCode);
    }

    [Fact]
    public async Task ManualUsage_LayoutEngine_FromDi()
    {
        // Sanity: AddDreport without MapDreportEndpoints still hands the engine
        // out via DI so users can plug it into their own controllers.
        var builder = new WebHostBuilder()
            .ConfigureServices(s => s.AddRouting().AddDreport())
            .Configure(app =>
            {
                app.UseRouting();
                app.UseEndpoints(e => e.MapGet("/custom",
                    (LayoutEngine engine) => Results.Json(new { count = engine.FontFamilyCount })));
            });
        using var server = new TestServer(builder);
        var client = server.CreateClient();
        var resp = await client.GetAsync("/custom");
        Assert.Equal(HttpStatusCode.OK, resp.StatusCode);
        var json = await resp.Content.ReadFromJsonAsync<JsonElement>();
        Assert.True(json.GetProperty("count").GetInt32() >= 1);
    }
}
