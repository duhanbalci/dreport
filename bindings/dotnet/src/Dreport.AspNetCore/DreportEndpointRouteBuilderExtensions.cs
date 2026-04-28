using System.Text.Json;
using Dreport.Service;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Routing;
using Microsoft.Extensions.DependencyInjection;

namespace Dreport.AspNetCore;

/// <summary>
/// Optional sugar for hosts that just want the editor's stock HTTP API.
/// Mirrors the original Rust/Axum backend endpoint contract 1:1, so the Vue
/// editor frontend does not need any code changes.
///
/// Skip this entirely if you prefer to wire endpoints by hand — the
/// <see cref="LayoutEngine"/> registered by <c>AddDreport()</c> is fully
/// usable from your own controllers / minimal API handlers.
/// </summary>
public static class DreportEndpointRouteBuilderExtensions
{
    /// <summary>
    /// Mount the dreport HTTP API under the given prefix (defaults to <c>/api</c>).
    /// Routes added:
    /// <list type="bullet">
    ///   <item><description><c>GET  {prefix}/health</c></description></item>
    ///   <item><description><c>POST {prefix}/render</c> — body <c>{ template, data }</c> → <c>application/pdf</c></description></item>
    ///   <item><description><c>POST {prefix}/layout</c> — body <c>{ template, data }</c> → LayoutResult JSON</description></item>
    ///   <item><description><c>GET  {prefix}/fonts</c> — registered families</description></item>
    ///   <item><description><c>GET  {prefix}/fonts/{family}/{weight}/{italic}</c> — raw font bytes</description></item>
    /// </list>
    /// </summary>
    public static IEndpointRouteBuilder MapDreportEndpoints(
        this IEndpointRouteBuilder builder,
        string prefix = "/api")
    {
        var p = prefix.TrimEnd('/');

        builder.MapGet($"{p}/health", () => Results.Json(new { status = "ok", version = typeof(LayoutEngine).Assembly.GetName().Version?.ToString() ?? "unknown" }));

        builder.MapPost($"{p}/render", async (HttpContext ctx, LayoutEngine engine) =>
        {
            var (template, data) = await ReadBodyAsync(ctx);
            try
            {
                var pdf = await Task.Run(() => engine.RenderPdf(template, data));
                return Results.File(pdf, "application/pdf");
            }
            catch (DreportException ex)
            {
                return MapError(ex);
            }
        });

        builder.MapPost($"{p}/layout", async (HttpContext ctx, LayoutEngine engine) =>
        {
            var (template, data) = await ReadBodyAsync(ctx);
            try
            {
                var layoutJson = await Task.Run(() => engine.ComputeLayout(template, data));
                return Results.Content(layoutJson, "application/json");
            }
            catch (DreportException ex)
            {
                return MapError(ex);
            }
        });

        builder.MapGet($"{p}/fonts", (LayoutEngine engine) =>
            Results.Json(engine.ListFontFamilies().Select(f => new
            {
                family = f.Family,
                variants = f.Variants.Select(v => new { weight = v.Weight, italic = v.Italic }),
            })));

        builder.MapGet($"{p}/fonts/{{family}}/{{weight}}/{{italic}}",
            (string family, ushort weight, string italic, LayoutEngine engine) =>
            {
                var isItalic = italic.Equals("true", StringComparison.OrdinalIgnoreCase) || italic == "1";
                var bytes = engine.GetFontBytes(family, weight, isItalic);
                return bytes is null
                    ? Results.NotFound($"Font bulunamadı: {family} weight={weight} italic={isItalic}")
                    : Results.File(bytes, "font/ttf");
            });

        return builder;
    }

    private static async Task<(string Template, string Data)> ReadBodyAsync(HttpContext ctx)
    {
        using var doc = await JsonDocument.ParseAsync(ctx.Request.Body);
        var root = doc.RootElement;
        var template = root.GetProperty("template").GetRawText();
        var data = root.TryGetProperty("data", out var d) ? d.GetRawText() : "{}";
        return (template, data);
    }

    private static IResult MapError(DreportException ex) => ex switch
    {
        InvalidTemplateException or Dreport.Service.InvalidDataException => Results.BadRequest(ex.Message),
        _ => Results.Problem(ex.Message, statusCode: StatusCodes.Status500InternalServerError),
    };
}
