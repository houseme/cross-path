use crate::parser::ParsedPath;
use crate::{PathConfig, PathResult, PathStyle};
use std::fmt;
use std::fmt::Write;

/// Path formatter for generating styled path strings
#[derive(Debug, Clone)]
pub struct PathFormatter {
    config: PathConfig,
}

impl PathFormatter {
    /// Create new path formatter
    #[must_use]
    pub fn new(config: &PathConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Format parsed path with specified style
    ///
    /// # Errors
    ///
    /// Returns `PathError` if formatting fails (e.g., invalid components).
    pub fn format(&self, parsed: &ParsedPath, target_style: PathStyle) -> PathResult<String> {
        match target_style {
            PathStyle::Windows => self.format_windows(parsed),
            PathStyle::Unix => self.format_unix(parsed),
            PathStyle::Auto => {
                let current_style = super::platform::current_style();
                self.format(parsed, current_style)
            }
        }
    }

    /// Format as Windows path
    fn format_windows(&self, parsed: &ParsedPath) -> PathResult<String> {
        if parsed.is_unc {
            return Ok(Self::format_unc_windows(parsed));
        }

        let mut result = String::new();

        // Add drive letter
        if let Some(drive) = parsed.drive_letter {
            let _ = write!(result, "{drive}:");
        } else if parsed.is_absolute {
            // Default drive
            result.push_str("C:");
        }

        // Add separator
        if parsed.is_absolute && !parsed.is_unc {
            result.push('\\');
        }

        // Add components
        for (i, component) in parsed.components.iter().enumerate() {
            if i > 0 {
                result.push('\\');
            }
            result.push_str(component);
        }

        // Normalize if requested
        if self.config.normalize {
            result = Self::normalize_windows_path(&result);
        }

        Ok(result)
    }

    /// Format as Unix path
    fn format_unix(&self, parsed: &ParsedPath) -> PathResult<String> {
        if parsed.is_unc {
            return Ok(Self::format_unc_unix(parsed));
        }

        let mut result = String::new();

        // UNC path handling
        if parsed.is_unc {
            if let (Some(server), Some(share)) = (&parsed.server, &parsed.share) {
                let _ = write!(result, "//{server}/{share}");
            }
        } else if parsed.is_absolute {
            if parsed.has_drive {
                // Map drive letter to Unix mount point
                if let Some(drive) = parsed.drive_letter {
                    let drive_lower = drive.to_ascii_lowercase();
                    result.push_str(&self.map_drive_to_unix(&format!("{drive_lower}:")));
                }
            } else {
                result.push('/');
            }
        }

        // Add components
        for component in &parsed.components {
            if !result.ends_with('/') && !result.is_empty() {
                result.push('/');
            }
            result.push_str(component);
        }

        // Normalize if requested
        if self.config.normalize {
            result = Self::normalize_unix_path(&result);
        }

        Ok(result)
    }

    /// Format UNC path as Windows format
    fn format_unc_windows(parsed: &ParsedPath) -> String {
        let mut result = String::from(r"\\");

        if let Some(server) = &parsed.server {
            result.push_str(server);
        }

        result.push('\\');

        if let Some(share) = &parsed.share {
            result.push_str(share);
        }

        for component in &parsed.components {
            result.push('\\');
            result.push_str(component);
        }

        result
    }

    /// Format UNC path as Unix format
    fn format_unc_unix(parsed: &ParsedPath) -> String {
        let mut result = String::from("//");

        if let Some(server) = &parsed.server {
            result.push_str(server);
        }

        result.push('/');

        if let Some(share) = &parsed.share {
            result.push_str(share);
        }

        for component in &parsed.components {
            result.push('/');
            result.push_str(component);
        }

        result
    }

    /// Map Windows drive letter to Unix path
    fn map_drive_to_unix(&self, drive: &str) -> String {
        for (windows_drive, unix_mount) in &self.config.drive_mappings {
            if windows_drive == drive {
                return unix_mount.clone();
            }
        }

        // Default mapping
        let drive_letter = drive.chars().next().unwrap().to_ascii_lowercase();
        format!("/mnt/{drive_letter}")
    }

    /// Normalize Windows path string
    fn normalize_windows_path(path: &str) -> String {
        let mut result = path.to_string();

        // Unify separators
        result = result.replace('/', "\\");

        // Remove duplicate separators
        while result.contains("\\\\") && !result.starts_with(r"\\") {
            result = result.replace("\\\\", "\\");
        }

        // Remove trailing separator (unless root path)
        if result.ends_with('\\') && result.len() > 3 && !result.starts_with(r"\\") {
            result.pop();
        }

        result
    }

    /// Normalize Unix path string
    fn normalize_unix_path(path: &str) -> String {
        let mut result = path.to_string();

        // Unify separators
        result = result.replace('\\', "/");

        // Remove duplicate separators
        while result.contains("//") && !result.starts_with("//") {
            result = result.replace("//", "/");
        }

        // Remove trailing separator (unless root path)
        if result.ends_with('/') && result != "/" && !result.starts_with("//") {
            result.pop();
        }

        result
    }
}

impl fmt::Display for PathFormatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PathFormatter(config: {:?})", self.config)
    }
}
