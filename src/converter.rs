use crate::{PathConfig, PathError, PathResult, PathStyle};
use regex::Regex;

/// Path converter for Windows ↔ Unix conversion
#[derive(Debug, Clone)]
pub struct PathConverter {
    config: PathConfig,
    windows_path_regex: Regex,
    unix_path_regex: Regex,
    drive_letter_regex: Regex,
}

impl PathConverter {
    /// Create new path converter
    ///
    /// # Panics
    ///
    /// Panics if the internal regex patterns are invalid.
    #[must_use]
    pub fn new(config: &PathConfig) -> Self {
        Self {
            config: config.clone(),
            windows_path_regex: Regex::new(r"^([a-zA-Z]:)([/\\].*)?$").unwrap(),
            unix_path_regex: Regex::new(r"^/([^/].*)?$").unwrap(),
            drive_letter_regex: Regex::new(r"^[a-zA-Z]:$").unwrap(),
        }
    }

    /// Convert path to specified style
    ///
    /// # Errors
    ///
    /// Returns `PathError` if the path cannot be converted or the format is unsupported.
    pub fn convert(&self, path: &str, target_style: PathStyle) -> PathResult<String> {
        let source_style = self.detect_style(path)?;

        if source_style == target_style {
            return Ok(path.to_string());
        }

        match (source_style, target_style) {
            (PathStyle::Windows, PathStyle::Unix) => self.windows_to_unix(path),
            (PathStyle::Unix, PathStyle::Windows) => self.unix_to_windows(path),
            _ => Err(PathError::UnsupportedFormat(format!(
                "Unsupported conversion: {source_style:?} -> {target_style:?}"
            ))),
        }
    }

    /// Detect path style
    ///
    /// # Errors
    ///
    /// Returns `PathError` if detection fails (though currently it always succeeds or returns default).
    pub fn detect_style(&self, path: &str) -> PathResult<PathStyle> {
        // Check for Windows path
        if self.windows_path_regex.is_match(path) {
            return Ok(PathStyle::Windows);
        }

        // Check for Unix path
        if self.unix_path_regex.is_match(path) {
            return Ok(PathStyle::Unix);
        }

        // Relative path, detect by separator
        if path.contains('\\') && !path.contains('/') {
            Ok(PathStyle::Windows)
        } else if path.contains('/') && !path.contains('\\') {
            Ok(PathStyle::Unix)
        } else {
            // Mixed separators, try intelligent detection
            if path.starts_with(r"\\") || path.contains(":\\") {
                Ok(PathStyle::Windows)
            } else if path.starts_with('/') {
                Ok(PathStyle::Unix)
            } else {
                // Default to current platform style
                Ok(super::platform::current_style())
            }
        }
    }

    /// Convert Windows path to Unix
    fn windows_to_unix(&self, path: &str) -> PathResult<String> {
        let normalized = self.normalize_windows_path(path);

        // Handle UNC paths
        if normalized.starts_with(r"\\") {
            return self.convert_unc_path(&normalized);
        }

        // Handle drive letter paths
        if let Some((drive, rest)) = self.split_drive_path(&normalized) {
            let unix_path = self.map_drive_to_unix(&drive, &rest);
            return Ok(unix_path);
        }

        // Handle relative paths
        let unix_path = normalized.replace('\\', "/");
        Ok(unix_path)
    }

    /// Convert Unix path to Windows
    fn unix_to_windows(&self, path: &str) -> PathResult<String> {
        let normalized = self.normalize_unix_path(path);

        // Check for mapped drive paths
        for (unix_prefix, windows_drive) in &self.config.drive_mappings {
            if normalized.starts_with(unix_prefix) {
                let rest = &normalized[unix_prefix.len()..];
                let windows_path = format!("{}{}", windows_drive, rest.replace('/', "\\"));
                return Ok(windows_path);
            }
        }

        // Handle regular Unix paths
        if normalized.starts_with('/') {
            // For absolute paths, map to default drive
            let windows_path = format!("C:{}", normalized.replace('/', "\\"));
            return Ok(windows_path);
        }

        // Relative paths
        let windows_path = normalized.replace('/', "\\");
        Ok(windows_path)
    }

    /// Normalize Windows path
    fn normalize_windows_path(&self, path: &str) -> String {
        let mut result = path.to_string();

        // Unify separators
        result = result.replace('/', "\\");

        // Remove duplicate separators
        while result.contains("\\\\") && !result.starts_with(r"\\") {
            result = result.replace("\\\\", "\\");
        }

        // Remove trailing separator (unless root path)
        if result.ends_with('\\') && result.len() > 3 && !self.drive_letter_regex.is_match(&result)
        {
            result.pop();
        }

        result
    }

    /// Normalize Unix path
    fn normalize_unix_path(&self, path: &str) -> String {
        let mut result = path.to_string();

        // Unify separators
        result = result.replace('\\', "/");

        // Remove duplicate separators
        while result.contains("//") {
            result = result.replace("//", "/");
        }

        // Remove trailing separator (unless root path)
        if result.ends_with('/') && result != "/" {
            result.pop();
        }

        result
    }

    /// Split drive letter from path
    fn split_drive_path(&self, path: &str) -> Option<(String, String)> {
        if path.len() >= 2 {
            let drive = &path[..2];
            if self.drive_letter_regex.is_match(drive) {
                let rest = if path.len() > 2 { &path[2..] } else { "" };
                return Some((drive.to_string(), rest.to_string()));
            }
        }
        None
    }

    /// Map Windows drive letter to Unix path
    fn map_drive_to_unix(&self, drive: &str, rest: &str) -> String {
        // Look for mapping configuration
        for (windows_drive, unix_mount) in &self.config.drive_mappings {
            if windows_drive == drive {
                return format!("{}{}", unix_mount, rest.replace('\\', "/"));
            }
        }

        // Default mapping
        let drive_letter = drive.chars().next().unwrap().to_ascii_lowercase();
        format!("/mnt/{}{}", drive_letter, rest.replace('\\', "/"))
    }

    /// Convert UNC path
    fn convert_unc_path(&self, path: &str) -> PathResult<String> {
        // UNC path format: \\server\share\path
        let parts: Vec<&str> = path.split('\\').collect();
        if parts.len() >= 4 {
            let server = parts[2];
            let share = parts[3];
            let rest = if parts.len() > 4 {
                parts[4..].join("/")
            } else {
                String::new()
            };
            let unix_path = format!("//{server}/{share}/{rest}");
            return Ok(unix_path.trim_end_matches('/').to_string());
        }

        Err(PathError::ParseError(format!("Invalid UNC path: {path}")))
    }
}
