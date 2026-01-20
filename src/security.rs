use crate::{PathError, PathResult};
use regex::Regex;
use std::path::Path;

/// Path security checker for preventing path-based attacks
#[derive(Debug, Clone)]
pub struct PathSecurityChecker {
    path_traversal_regex: Regex,
    dangerous_patterns: Vec<Regex>,
    reserved_names: Vec<&'static str>,
}

impl Default for PathSecurityChecker {
    fn default() -> Self {
        Self {
            path_traversal_regex: Regex::new(r"(\.\./|\.\.\\)").unwrap(),
            dangerous_patterns: vec![
                Regex::new(r"(?i)\.(exe|bat|cmd|sh|php|py|js)$").unwrap(),
                Regex::new(r"^/proc/").unwrap(),
                Regex::new(r"^/dev/").unwrap(),
                Regex::new(r"^/sys/").unwrap(),
            ],
            reserved_names: vec![
                "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
                "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8",
                "LPT9",
            ],
        }
    }
}

impl PathSecurityChecker {
    /// Create new security checker
    pub fn new() -> Self {
        Self::default()
    }

    /// Check path security (static method)
    pub fn check_path_security(path: &Path) -> PathResult<bool> {
        let checker = Self::new();
        checker.check(path)
    }

    /// Perform security checks on path
    pub fn check(&self, path: &Path) -> PathResult<bool> {
        // Check for path traversal attacks
        if self.detect_path_traversal(path) {
            return Err(PathError::security_error("Path traversal attack detected"));
        }

        // Check for dangerous patterns
        if self.contains_dangerous_patterns(path) {
            return Err(PathError::security_error(
                "Path contains dangerous patterns",
            ));
        }

        // Check for reserved names (Windows)
        if self.contains_reserved_names(path) {
            return Err(PathError::security_error(
                "Path contains Windows reserved names",
            ));
        }

        // Check for system directory access attempts
        if self.accesses_system_directories(path) {
            return Err(PathError::security_error(
                "Attempt to access system directories",
            ));
        }

        Ok(true)
    }

    /// Detect path traversal patterns
    fn detect_path_traversal(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.path_traversal_regex.is_match(&path_str)
    }

    /// Check for dangerous file patterns
    fn contains_dangerous_patterns(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.dangerous_patterns
            .iter()
            .any(|re| re.is_match(&path_str))
    }

    /// Check for Windows reserved names
    fn contains_reserved_names(&self, path: &Path) -> bool {
        #[cfg(target_os = "windows")]
        {
            if let Some(file_name) = path.file_name() {
                let name = file_name.to_string_lossy();
                let name_without_ext = name.split('.').next().unwrap_or("");
                self.reserved_names
                    .iter()
                    .any(|&reserved| name_without_ext.eq_ignore_ascii_case(reserved))
            } else {
                false
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            // On non-Windows systems, we generally don't need to check for Windows reserved names
            // unless we are specifically validating for cross-platform compatibility.
            // For now, we skip this check to avoid false positives on valid Unix filenames.
            let _ = path; // Suppress unused variable warning
            false
        }
    }

    /// Check if path attempts to access system directories
    fn accesses_system_directories(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        #[cfg(target_os = "windows")]
        {
            let system_dirs = vec![
                r"C:\Windows",
                r"C:\System32",
                r"C:\Program Files",
                r"C:\ProgramData",
            ];
            system_dirs.iter().any(|&dir| path_str.starts_with(dir))
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Common Unix system directories
            // Covers Linux, macOS, FreeBSD, OpenBSD, Android, etc.
            let system_dirs = vec![
                "/bin",
                "/sbin",
                "/usr/bin",
                "/usr/sbin",
                "/etc",
                "/root",
                "/var",
                "/lib",
                "/boot",
                "/dev",
                "/proc",
                "/sys",
            ];

            // Android specific system directories
            #[cfg(target_os = "android")]
            let system_dirs = {
                let mut dirs = system_dirs;
                dirs.extend_from_slice(&[
                    "/system",
                    "/data",
                    "/cache",
                    "/vendor",
                    "/oem",
                    "/odm",
                ]);
                dirs
            };

            // macOS specific system directories
            #[cfg(target_os = "macos")]
            let system_dirs = {
                let mut dirs = system_dirs;
                dirs.extend_from_slice(&[
                    "/System",
                    "/Library",
                    "/private",
                    "/Volumes",
                    "/Network",
                ]);
                dirs
            };

            system_dirs.iter().any(|&dir| path_str.starts_with(dir))
        }
    }

    /// Sanitize path by removing dangerous characters
    pub fn sanitize_path(path: &str) -> String {
        let mut sanitized = path.to_string();

        // Remove path traversal sequences
        sanitized = sanitized.replace("../", "").replace("..\\", "");

        // Remove dangerous characters
        let dangerous = ['<', '>', ':', '"', '|', '?', '*', '\\', '/', '\0'];
        for c in dangerous {
            sanitized = sanitized.replace(c, "_");
        }

        // Limit path length
        if sanitized.len() > 255 {
            sanitized = sanitized[..255].to_string();
        }

        sanitized
    }
}
