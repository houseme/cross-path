use std::fmt;

/// Path error type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathError {
    /// Invalid path
    InvalidPath(String),

    /// Encoding error
    EncodingError(String),

    /// Security error
    SecurityError(String),

    /// Platform-specific error
    PlatformError(String),

    /// Path normalization error
    NormalizationError(String),

    /// Path parsing error
    ParseError(String),

    /// IO error
    IoError(String),

    /// Unsupported path format
    UnsupportedFormat(String),

    /// Drive mapping error
    DriveMappingError(String),
}

impl PathError {
    /// Create new `InvalidPath` error
    pub fn invalid_path(msg: impl Into<String>) -> Self {
        Self::InvalidPath(msg.into())
    }

    /// Create new `EncodingError`
    pub fn encoding_error(msg: impl Into<String>) -> Self {
        Self::EncodingError(msg.into())
    }

    /// Create new `SecurityError`
    pub fn security_error(msg: impl Into<String>) -> Self {
        Self::SecurityError(msg.into())
    }

    /// Create new `PlatformError`
    pub fn platform_error(msg: impl Into<String>) -> Self {
        Self::PlatformError(msg.into())
    }
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath(msg) => write!(f, "Invalid path: {msg}"),
            Self::EncodingError(msg) => write!(f, "Encoding error: {msg}"),
            Self::SecurityError(msg) => write!(f, "Security error: {msg}"),
            Self::PlatformError(msg) => write!(f, "Platform error: {msg}"),
            Self::NormalizationError(msg) => write!(f, "Normalization error: {msg}"),
            Self::ParseError(msg) => write!(f, "Parse error: {msg}"),
            Self::IoError(msg) => write!(f, "IO error: {msg}"),
            Self::UnsupportedFormat(msg) => write!(f, "Unsupported format: {msg}"),
            Self::DriveMappingError(msg) => write!(f, "Drive mapping error: {msg}"),
        }
    }
}

impl std::error::Error for PathError {}

impl From<std::io::Error> for PathError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}
