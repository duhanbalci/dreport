using Dreport.Service;
using Microsoft.Extensions.DependencyInjection;

namespace Dreport.AspNetCore;

/// <summary>
/// DI registration for <see cref="LayoutEngine"/>. Registers the engine as a
/// process-wide singleton so consumers can inject it into controllers,
/// endpoint handlers, background services, or test fixtures.
/// </summary>
public static class DreportServiceCollectionExtensions
{
    /// <summary>
    /// Registers a singleton <see cref="LayoutEngine"/>. Once added, you can:
    /// <list type="bullet">
    ///   <item><description>Inject <see cref="LayoutEngine"/> into your own MVC controllers, minimal API handlers, or background services.</description></item>
    ///   <item><description>Call <c>app.MapDreportEndpoints()</c> to also mount the ready-made HTTP API the editor talks to.</description></item>
    /// </list>
    /// </summary>
    public static IServiceCollection AddDreport(
        this IServiceCollection services,
        Action<DreportOptions>? configure = null)
    {
        var options = new DreportOptions();
        configure?.Invoke(options);

        services.AddSingleton(options);
        services.AddSingleton(_ => CreateEngine(options));
        return services;
    }

    private static LayoutEngine CreateEngine(DreportOptions options)
    {
        var engine = options.LoadEmbeddedFonts ? new LayoutEngine() : LayoutEngine.CreateEmpty();
        if (!string.IsNullOrEmpty(options.FontsDirectory) && Directory.Exists(options.FontsDirectory))
        {
            engine.RegisterFontsDirectory(options.FontsDirectory);
        }
        return engine;
    }
}
