use crate::{PathError, PathResult};
use encoding_rs::{UTF_16LE, UTF_8, WINDOWS_1252};

/// Unicode encoding handler for path strings
#[derive(Debug, Clone, Copy)]
pub struct UnicodeHandler;

impl UnicodeHandler {
    /// Detect string encoding
    #[must_use] 
    pub fn detect_encoding(bytes: &[u8]) -> &'static encoding_rs::Encoding {
        // Simple UTF-8 detection
        if String::from_utf8(bytes.to_vec()).is_ok() {
            return UTF_8;
        }

        // Try to detect UTF-16 LE (BOM)
        if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
            return UTF_16LE;
        }

        // Default to Windows-1252 (common Windows encoding)
        WINDOWS_1252
    }

    /// Convert bytes to UTF-8 string
    pub fn convert_to_utf8(bytes: &[u8]) -> PathResult<String> {
        let encoding = Self::detect_encoding(bytes);
        let (decoded, _, had_errors) = encoding.decode(bytes);

        if had_errors {
            return Err(PathError::encoding_error(
                "Encoding conversion encountered errors",
            ));
        }

        Ok(decoded.into_owned())
    }

    /// Convert UTF-8 string to target encoding bytes
    pub fn convert_from_utf8(
        text: &str,
        target_encoding: &'static encoding_rs::Encoding,
    ) -> PathResult<Vec<u8>> {
        let (encoded, _, had_errors) = target_encoding.encode(text);

        if had_errors {
            return Err(PathError::encoding_error(
                "Encoding conversion encountered errors",
            ));
        }

        Ok(encoded.into_owned())
    }

    /// Normalize Windows path by removing invalid characters
    #[must_use] 
    pub fn normalize_windows_path(path: &str) -> String {
        let mut result = path.to_string();

        // Replace Windows-disallowed characters
        let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
        for c in invalid_chars {
            result = result.replace(c, "_");
        }

        // Remove control characters
        result = result.chars().filter(|c| !c.is_control()).collect();

        result
    }

    /// Normalize Unix path by removing invalid characters
    #[must_use] 
    pub fn normalize_unix_path(path: &str) -> String {
        let mut result = path.to_string();

        // Unix paths disallow null characters
        result = result.replace('\0', "");

        // Remove control characters
        result = result.chars().filter(|c| !c.is_control()).collect();

        result
    }

    /// Convert path encoding (mainly for Windows non-UTF-8 encodings)
    pub fn convert_path_encoding(
        path: &str,
        from: &'static encoding_rs::Encoding,
        to: &'static encoding_rs::Encoding,
    ) -> PathResult<String> {
        if from == to {
            return Ok(path.to_string());
        }

        // Encode then decode
        let bytes = from.encode(path).0;
        let (decoded, _, had_errors) = to.decode(&bytes);

        if had_errors {
            return Err(PathError::encoding_error("Path encoding conversion failed"));
        }

        Ok(decoded.into_owned())
    }
}
