namespace Dreport.AspNetCore;

/// <summary>
/// Configuration for the dreport ASP.NET Core integration.
/// </summary>
public sealed class DreportOptions
{
    /// <summary>
    /// Optional directory whose <c>.ttf</c> / <c>.otf</c> files are loaded into the
    /// engine on startup, in addition to the embedded default fonts.
    /// </summary>
    public string? FontsDirectory { get; set; }

    /// <summary>
    /// When <c>true</c> (default), embedded default fonts (Noto Sans, Noto Sans Mono)
    /// are registered. Set to <c>false</c> to start with an empty registry — useful
    /// when the host wants to provide a fully custom font set.
    /// </summary>
    public bool LoadEmbeddedFonts { get; set; } = true;
}
