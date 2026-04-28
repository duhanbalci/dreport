use thiserror::Error;

/// dreport-service üzerinden yapılan tüm operasyonların hata tipi.
/// FFI ve HTTP adapter'ları bu enum'u kendi error formatlarına map'ler.
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("invalid template JSON: {0}")]
    InvalidTemplateJson(String),

    #[error("invalid data JSON: {0}")]
    InvalidDataJson(String),

    #[error("font parse failed: bytes do not contain a valid TTF/OTF face")]
    FontParseFailed,

    #[error("font directory not found: {0}")]
    FontDirNotFound(String),

    #[error("font directory read error: {0}")]
    FontDirRead(String),

    #[error("layout computation failed: {0}")]
    LayoutFailed(String),

    #[error("pdf rendering failed: {0}")]
    PdfFailed(String),

    #[error("layout result serialization failed: {0}")]
    SerializationFailed(String),
}

impl ServiceError {
    /// Stable numeric code for FFI consumers.
    pub fn code(&self) -> i32 {
        match self {
            Self::InvalidTemplateJson(_) => 1,
            Self::InvalidDataJson(_) => 2,
            Self::FontParseFailed => 3,
            Self::FontDirNotFound(_) => 4,
            Self::FontDirRead(_) => 5,
            Self::LayoutFailed(_) => 6,
            Self::PdfFailed(_) => 7,
            Self::SerializationFailed(_) => 8,
        }
    }
}

pub type ServiceResult<T> = Result<T, ServiceError>;
