using System.Text.Json;
using Dreport.Service;
using Xunit;

namespace Dreport.Service.Tests;

public class LayoutEngineTests
{
    private const string Template = """
        {
          "id": "csharp",
          "name": "C# Test",
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
                "content": "Hello from C#"
              }
            ]
          }
        }
        """;
    private const string Data = "{}";

    private static byte[] LoadFixtureFont() =>
        File.ReadAllBytes(Path.Combine(AppContext.BaseDirectory, "fixtures", "NotoSans-Regular.ttf"));

    // ---------------------------------------------------------------------
    // Lifecycle
    // ---------------------------------------------------------------------

    [Fact]
    public void Construct_DefaultEngine_HasEmbeddedFonts()
    {
        using var engine = new LayoutEngine();
        Assert.True(engine.FontFamilyCount >= 1);
    }

    [Fact]
    public void CreateEmpty_StartsWithoutFonts()
    {
        using var engine = LayoutEngine.CreateEmpty();
        Assert.Equal(0, engine.FontFamilyCount);
    }

    [Fact]
    public void NativeVersion_ReturnsNonEmpty()
    {
        var v = LayoutEngine.NativeVersion;
        Assert.False(string.IsNullOrEmpty(v));
        Assert.Contains('.', v);
    }

    [Fact]
    public void Dispose_TwiceIsSafe()
    {
        var engine = new LayoutEngine();
        engine.Dispose();
        engine.Dispose();
    }

    [Fact]
    public void Operations_AfterDispose_Throw()
    {
        var engine = new LayoutEngine();
        engine.Dispose();
        Assert.Throws<ObjectDisposedException>(() => engine.RenderPdf(Template, Data));
        Assert.Throws<ObjectDisposedException>(() => engine.ListFontFamilies());
    }

    // ---------------------------------------------------------------------
    // Font registry
    // ---------------------------------------------------------------------

    [Fact]
    public void RegisterFont_ValidBytes_IncreasesCount()
    {
        using var engine = LayoutEngine.CreateEmpty();
        engine.RegisterFont(LoadFixtureFont());
        Assert.Equal(1, engine.FontFamilyCount);
    }

    [Fact]
    public void RegisterFont_InvalidBytes_ThrowsFontParseException()
    {
        using var engine = LayoutEngine.CreateEmpty();
        var ex = Assert.Throws<FontParseException>(() =>
            engine.RegisterFont(new byte[] { 1, 2, 3, 4 }));
        Assert.Equal(Native.ERR_FONT_PARSE_FAILED, ex.Code);
    }

    [Fact]
    public void RegisterFontsDirectory_NonExisting_ThrowsFontDirectoryNotFound()
    {
        using var engine = LayoutEngine.CreateEmpty();
        Assert.Throws<FontDirectoryNotFoundException>(() =>
            engine.RegisterFontsDirectory("/no/such/dreport/test/path/xyz"));
    }

    [Fact]
    public void RegisterFontsDirectory_ValidDir_LoadsCount()
    {
        var fixturesDir = Path.Combine(AppContext.BaseDirectory, "fixtures");
        Assert.True(Directory.Exists(fixturesDir));

        using var engine = LayoutEngine.CreateEmpty();
        var count = engine.RegisterFontsDirectory(fixturesDir);
        Assert.True(count >= 1);
    }

    [Fact]
    public void GetFontBytes_KnownVariant_ReturnsBytes()
    {
        using var engine = new LayoutEngine();
        var bytes = engine.GetFontBytes("Noto Sans", 400, false);
        Assert.NotNull(bytes);
        Assert.True(bytes!.Length > 1000);
    }

    [Fact]
    public void GetFontBytes_UnknownVariant_ReturnsNull()
    {
        using var engine = new LayoutEngine();
        Assert.Null(engine.GetFontBytes("DoesNotExist", 400, false));
    }

    [Fact]
    public void ListFontFamilies_ContainsNotoSans()
    {
        using var engine = new LayoutEngine();
        var families = engine.ListFontFamilies();
        Assert.Contains(families, f => f.Family.ToLowerInvariant().Contains("noto"));
    }

    // ---------------------------------------------------------------------
    // Render pipeline
    // ---------------------------------------------------------------------

    [Fact]
    public void ComputeLayout_ValidInputs_ReturnsLayoutJson()
    {
        using var engine = new LayoutEngine();
        var json = engine.ComputeLayout(Template, Data);
        using var doc = JsonDocument.Parse(json);
        Assert.True(doc.RootElement.TryGetProperty("pages", out var pages));
        Assert.Equal(JsonValueKind.Array, pages.ValueKind);
        Assert.True(pages.GetArrayLength() >= 1);
    }

    [Fact]
    public void ComputeLayout_InvalidTemplate_ThrowsInvalidTemplate()
    {
        using var engine = new LayoutEngine();
        Assert.Throws<InvalidTemplateException>(() => engine.ComputeLayout("{not json", Data));
    }

    [Fact]
    public void ComputeLayout_InvalidData_ThrowsInvalidData()
    {
        using var engine = new LayoutEngine();
        Assert.Throws<InvalidDataException>(() => engine.ComputeLayout(Template, "{not json"));
    }

    [Fact]
    public void RenderPdf_ValidInputs_ReturnsPdfBytes()
    {
        using var engine = new LayoutEngine();
        var pdf = engine.RenderPdf(Template, Data);
        Assert.True(pdf.Length > 100);
        Assert.Equal((byte)'%', pdf[0]);
        Assert.Equal((byte)'P', pdf[1]);
        Assert.Equal((byte)'D', pdf[2]);
        Assert.Equal((byte)'F', pdf[3]);
    }

    [Fact]
    public void RenderPdf_InvalidTemplate_ThrowsInvalidTemplate()
    {
        using var engine = new LayoutEngine();
        Assert.Throws<InvalidTemplateException>(() => engine.RenderPdf("{not json", Data));
    }

    // ---------------------------------------------------------------------
    // Concurrency
    // ---------------------------------------------------------------------

    [Fact]
    public void RenderPdf_Parallel_ProducesPdfs()
    {
        using var engine = new LayoutEngine();
        var success = 0;
        Parallel.For(0, 16, _ =>
        {
            var pdf = engine.RenderPdf(Template, Data);
            if (pdf.Length > 100 && pdf[0] == (byte)'%')
            {
                Interlocked.Increment(ref success);
            }
        });
        Assert.Equal(16, success);
    }

    // ---------------------------------------------------------------------
    // Error code stability (matches Rust ServiceError::code() contract)
    // ---------------------------------------------------------------------

    [Fact]
    public void ErrorCode_InvalidTemplate_IsMinusOne()
    {
        var ex = new InvalidTemplateException("x");
        Assert.Equal(-1, ex.Code);
    }

    [Fact]
    public void ErrorCode_FontParseFailed_IsMinusThree()
    {
        var ex = new FontParseException("x");
        Assert.Equal(-3, ex.Code);
    }
}
