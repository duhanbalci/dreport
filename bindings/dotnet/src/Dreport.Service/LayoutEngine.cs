using System.Text;
using System.Text.Json;

namespace Dreport.Service;

/// <summary>
/// Managed wrapper around a single dreport native engine handle.
///
/// Thread-safe: every operation goes through the underlying Rust service which
/// uses internal locking. You can keep one process-wide instance and call
/// concurrent <see cref="RenderPdf"/> from any number of threads.
/// </summary>
public sealed class LayoutEngine : IDisposable
{
    private IntPtr _handle;
    private readonly object _disposeLock = new();
    private bool _disposed;

    /// <summary>Create an engine with the embedded default fonts loaded.</summary>
    public LayoutEngine() : this(Native.dreport_new())
    {
    }

    private LayoutEngine(IntPtr handle)
    {
        if (handle == IntPtr.Zero)
        {
            throw new InvalidOperationException("dreport_new returned a null handle");
        }
        _handle = handle;
    }

    /// <summary>Create an engine with no fonts pre-loaded.</summary>
    public static LayoutEngine CreateEmpty() => new(Native.dreport_new_empty());

    /// <summary>Native crate version, e.g. "0.2.0".</summary>
    public static string NativeVersion
    {
        get
        {
            var ptr = Native.dreport_version();
            return ptr == IntPtr.Zero ? string.Empty : System.Runtime.InteropServices.Marshal.PtrToStringAnsi(ptr) ?? string.Empty;
        }
    }

    // ---------------------------------------------------------------------
    // Font registry
    // ---------------------------------------------------------------------

    /// <summary>Number of distinct font families currently registered.</summary>
    public int FontFamilyCount
    {
        get
        {
            EnsureNotDisposed();
            var count = Native.dreport_font_family_count(_handle);
            if (count < 0)
            {
                throw DreportException.FromCode((int)count, "dreport_font_family_count failed");
            }
            return (int)count;
        }
    }

    /// <summary>Register a font from raw TTF/OTF bytes.</summary>
    public unsafe void RegisterFont(ReadOnlySpan<byte> data)
    {
        EnsureNotDisposed();
        if (data.IsEmpty)
        {
            throw new ArgumentException("font bytes empty", nameof(data));
        }
        fixed (byte* ptr = data)
        {
            var rc = Native.dreport_register_font(_handle, ptr, (nuint)data.Length);
            if (rc != Native.OK)
            {
                throw DreportException.FromCode(rc, "dreport_register_font failed");
            }
        }
    }

    /// <summary>Register every <c>.ttf</c>/<c>.otf</c> file in <paramref name="directory"/>.</summary>
    /// <returns>Number of fonts that registered successfully.</returns>
    public unsafe int RegisterFontsDirectory(string directory)
    {
        EnsureNotDisposed();
        ArgumentException.ThrowIfNullOrEmpty(directory);

        var bytes = Encoding.UTF8.GetBytes(directory);
        nuint count;
        int rc;
        fixed (byte* ptr = bytes)
        {
            rc = Native.dreport_register_fonts_dir(_handle, ptr, (nuint)bytes.Length, out count);
        }
        if (rc != Native.OK)
        {
            throw DreportException.FromCode(rc, $"dreport_register_fonts_dir failed for '{directory}'");
        }
        return (int)count;
    }

    /// <summary>Get raw bytes for a specific font variant. Returns null when unknown.</summary>
    public unsafe byte[]? GetFontBytes(string family, ushort weight, bool italic)
    {
        EnsureNotDisposed();
        ArgumentException.ThrowIfNullOrEmpty(family);

        var bytes = Encoding.UTF8.GetBytes(family);
        Native.DreportBuffer buffer;
        int rc;
        fixed (byte* ptr = bytes)
        {
            rc = Native.dreport_get_font_bytes(_handle, ptr, (nuint)bytes.Length, weight, italic, out buffer);
        }
        if (rc != Native.OK)
        {
            throw DreportException.FromCode(rc, "dreport_get_font_bytes failed");
        }
        var data = Native.ConsumeBuffer(buffer);
        return data.Length == 0 ? null : data;
    }

    /// <summary>List every registered font family with its variants.</summary>
    public IReadOnlyList<FontFamily> ListFontFamilies()
    {
        EnsureNotDisposed();
        var rc = Native.dreport_list_fonts_json(_handle, out var buffer);
        if (rc != Native.OK)
        {
            throw DreportException.FromCode(rc, "dreport_list_fonts_json failed");
        }
        var json = Native.ConsumeBuffer(buffer);
        if (json.Length == 0)
        {
            return Array.Empty<FontFamily>();
        }
        var families = JsonSerializer.Deserialize<List<FontFamily>>(json);
        return families ?? new List<FontFamily>();
    }

    // ---------------------------------------------------------------------
    // Render pipeline
    // ---------------------------------------------------------------------

    /// <summary>Compute layout from JSON inputs. Returns the LayoutResult JSON string.</summary>
    public unsafe string ComputeLayout(string templateJson, string dataJson)
    {
        EnsureNotDisposed();
        ArgumentException.ThrowIfNullOrEmpty(templateJson);
        ArgumentNullException.ThrowIfNull(dataJson);

        var tplBytes = Encoding.UTF8.GetBytes(templateJson);
        var dataBytes = Encoding.UTF8.GetBytes(dataJson);
        Native.DreportBuffer buffer;
        int rc;
        fixed (byte* tplPtr = tplBytes)
        fixed (byte* dataPtr = dataBytes)
        {
            rc = Native.dreport_compute_layout(
                _handle, tplPtr, (nuint)tplBytes.Length, dataPtr, (nuint)dataBytes.Length, out buffer);
        }
        if (rc != Native.OK)
        {
            throw DreportException.FromCode(rc, "dreport_compute_layout failed");
        }
        return Encoding.UTF8.GetString(Native.ConsumeBuffer(buffer));
    }

    /// <summary>Render a PDF document. Returns the raw PDF bytes.</summary>
    public unsafe byte[] RenderPdf(string templateJson, string dataJson)
    {
        EnsureNotDisposed();
        ArgumentException.ThrowIfNullOrEmpty(templateJson);
        ArgumentNullException.ThrowIfNull(dataJson);

        var tplBytes = Encoding.UTF8.GetBytes(templateJson);
        var dataBytes = Encoding.UTF8.GetBytes(dataJson);
        Native.DreportBuffer buffer;
        int rc;
        fixed (byte* tplPtr = tplBytes)
        fixed (byte* dataPtr = dataBytes)
        {
            rc = Native.dreport_render_pdf(
                _handle, tplPtr, (nuint)tplBytes.Length, dataPtr, (nuint)dataBytes.Length, out buffer);
        }
        if (rc != Native.OK)
        {
            throw DreportException.FromCode(rc, "dreport_render_pdf failed");
        }
        return Native.ConsumeBuffer(buffer);
    }

    // ---------------------------------------------------------------------
    // Disposal
    // ---------------------------------------------------------------------

    public void Dispose()
    {
        lock (_disposeLock)
        {
            if (_disposed) return;
            _disposed = true;
            if (_handle != IntPtr.Zero)
            {
                Native.dreport_free(_handle);
                _handle = IntPtr.Zero;
            }
        }
    }

    private void EnsureNotDisposed()
    {
        if (_disposed)
        {
            throw new ObjectDisposedException(nameof(LayoutEngine));
        }
    }
}
