// P/Invoke surface for libdreport_ffi. Mirrors dreport-ffi/include/dreport.h
// 1:1. Higher-level wrappers live in LayoutEngine.cs.

using System.Runtime.InteropServices;

namespace Dreport.Service;

internal static class Native
{
    // The shared library is named libdreport_ffi.{dylib,so} or dreport_ffi.dll.
    // .NET's runtime resolves it via the runtimes/<rid>/native/ pattern when the
    // package is consumed; for local development the file lives next to the test
    // assembly under bin/<config>/<tfm>/runtimes/<rid>/native/.
    internal const string Lib = "dreport_ffi";

    // ----- Return codes (mirror dreport_ffi::error_code) -------------------

    public const int OK = 0;
    public const int NULL_HANDLE = -100;
    public const int NULL_POINTER = -101;
    public const int INVALID_UTF8 = -102;
    public const int PANIC = -103;

    // Service-level error codes are returned as the negation of
    // ServiceError::code(), e.g. FontParseFailed (3) → -3.
    public const int ERR_INVALID_TEMPLATE_JSON = -1;
    public const int ERR_INVALID_DATA_JSON = -2;
    public const int ERR_FONT_PARSE_FAILED = -3;
    public const int ERR_FONT_DIR_NOT_FOUND = -4;
    public const int ERR_FONT_DIR_READ = -5;
    public const int ERR_LAYOUT_FAILED = -6;
    public const int ERR_PDF_FAILED = -7;
    public const int ERR_SERIALIZATION_FAILED = -8;

    // ----- ByteBuffer ------------------------------------------------------

    [StructLayout(LayoutKind.Sequential)]
    public struct DreportBuffer
    {
        public IntPtr Data;
        public nuint Len;
        public nuint Cap;

        public static DreportBuffer Empty => default;
    }

    // ----- Lifecycle -------------------------------------------------------

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr dreport_new();

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr dreport_new_empty();

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void dreport_free(IntPtr handle);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void dreport_buffer_free(DreportBuffer buffer);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr dreport_version();

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern int dreport_last_error(out DreportBuffer buffer);

    // ----- Font registry ---------------------------------------------------

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern unsafe int dreport_register_font(IntPtr handle, byte* data, nuint len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern unsafe int dreport_register_fonts_dir(
        IntPtr handle,
        byte* path,
        nuint pathLen,
        out nuint outCount);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern int dreport_list_fonts_json(IntPtr handle, out DreportBuffer outBuffer);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern unsafe int dreport_get_font_bytes(
        IntPtr handle,
        byte* family,
        nuint familyLen,
        ushort weight,
        [MarshalAs(UnmanagedType.U1)] bool italic,
        out DreportBuffer outBuffer);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern nint dreport_font_family_count(IntPtr handle);

    // ----- Render pipeline -------------------------------------------------

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern unsafe int dreport_compute_layout(
        IntPtr handle,
        byte* template_,
        nuint templateLen,
        byte* data,
        nuint dataLen,
        out DreportBuffer outBuffer);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern unsafe int dreport_render_pdf(
        IntPtr handle,
        byte* template_,
        nuint templateLen,
        byte* data,
        nuint dataLen,
        out DreportBuffer outBuffer);

    // ----- Helpers ---------------------------------------------------------

    /// <summary>Copy a native buffer into a managed byte[] and free the native side.</summary>
    public static byte[] ConsumeBuffer(DreportBuffer buffer)
    {
        if (buffer.Data == IntPtr.Zero || buffer.Len == 0)
        {
            // Still free the buffer in case cap > 0 (defensive — current FFI never returns this).
            if (buffer.Cap > 0)
            {
                dreport_buffer_free(buffer);
            }
            return Array.Empty<byte>();
        }

        var bytes = new byte[buffer.Len];
        Marshal.Copy(buffer.Data, bytes, 0, (int)buffer.Len);
        dreport_buffer_free(buffer);
        return bytes;
    }

    /// <summary>Read the most recent FFI error message for the current thread.</summary>
    public static string GetLastError()
    {
        if (dreport_last_error(out var buffer) != OK)
        {
            return string.Empty;
        }
        var bytes = ConsumeBuffer(buffer);
        return bytes.Length == 0 ? string.Empty : System.Text.Encoding.UTF8.GetString(bytes);
    }
}
