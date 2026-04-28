namespace Dreport.Service;

/// <summary>
/// Thrown when the underlying dreport service returns an error. The numeric
/// <see cref="Code"/> mirrors the FFI return code (negative values).
/// </summary>
public class DreportException : Exception
{
    public int Code { get; }

    public DreportException(int code, string message) : base(message)
    {
        Code = code;
    }

    internal static DreportException FromCode(int code, string fallbackMessage)
    {
        var nativeMessage = Native.GetLastError();
        var message = string.IsNullOrEmpty(nativeMessage) ? fallbackMessage : nativeMessage;
        return code switch
        {
            Native.ERR_INVALID_TEMPLATE_JSON => new InvalidTemplateException(message),
            Native.ERR_INVALID_DATA_JSON => new InvalidDataException(message),
            Native.ERR_FONT_PARSE_FAILED => new FontParseException(message),
            Native.ERR_FONT_DIR_NOT_FOUND => new FontDirectoryNotFoundException(message),
            Native.ERR_FONT_DIR_READ => new FontDirectoryReadException(message),
            Native.ERR_LAYOUT_FAILED => new LayoutException(message),
            Native.ERR_PDF_FAILED => new PdfRenderException(message),
            _ => new DreportException(code, message),
        };
    }
}

public sealed class InvalidTemplateException : DreportException
{
    public InvalidTemplateException(string message) : base(Native.ERR_INVALID_TEMPLATE_JSON, message) { }
}

public sealed class InvalidDataException : DreportException
{
    public InvalidDataException(string message) : base(Native.ERR_INVALID_DATA_JSON, message) { }
}

public sealed class FontParseException : DreportException
{
    public FontParseException(string message) : base(Native.ERR_FONT_PARSE_FAILED, message) { }
}

public sealed class FontDirectoryNotFoundException : DreportException
{
    public FontDirectoryNotFoundException(string message) : base(Native.ERR_FONT_DIR_NOT_FOUND, message) { }
}

public sealed class FontDirectoryReadException : DreportException
{
    public FontDirectoryReadException(string message) : base(Native.ERR_FONT_DIR_READ, message) { }
}

public sealed class LayoutException : DreportException
{
    public LayoutException(string message) : base(Native.ERR_LAYOUT_FAILED, message) { }
}

public sealed class PdfRenderException : DreportException
{
    public PdfRenderException(string message) : base(Native.ERR_PDF_FAILED, message) { }
}
