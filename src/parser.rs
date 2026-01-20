use crate::PathResult;
use regex::Regex;
use std::path::{Path, PathBuf};

/// Path parser for analyzing path structure
#[derive(Debug, Clone)]
pub struct PathParser {
    windows_absolute_regex: Regex,
    unix_absolute_regex: Regex,
    unc_path_regex: Regex,
}

impl Default for PathParser {
    fn default() -> Self {
        Self {
            windows_absolute_regex: Regex::new(r"^[a-zA-Z]:[/\\].*$").unwrap(),
            unix_absolute_regex: Regex::new(r"^/.*$").unwrap(),
            unc_path_regex: Regex::new(r"^\\\\[^\\]+\\[^\\]+").unwrap(),
        }
    }
}

impl PathParser {
    /// Create new path parser
    #[must_use] 
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse path into structured components
    pub fn parse(path: &str) -> PathResult<ParsedPath> {
        let parser = Self::new();
        parser.parse_internal(path)
    }

    fn parse_internal(&self, path: &str) -> PathResult<ParsedPath> {
        let mut parsed = ParsedPath {
            original: path.to_string(),
            components: Vec::new(),
            is_absolute: false,
            has_drive: false,
            drive_letter: None,
            is_unc: false,
            server: None,
            share: None,
        };

        // Detect UNC path
        if self.unc_path_regex.is_match(path) {
            parsed.is_unc = true;
            if let Some((server, share)) = self.parse_unc_path(path) {
                parsed.server = Some(server);
                parsed.share = Some(share);
            }
            parsed.is_absolute = true;
            return Ok(parsed);
        }

        // Detect Windows absolute path
        if self.windows_absolute_regex.is_match(path) {
            parsed.is_absolute = true;
            parsed.has_drive = true;
            parsed.drive_letter = Some(path.chars().next().unwrap().to_ascii_uppercase());

            // Parse components
            let normalized = path.replace('\\', "/");
            let components: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
            parsed.components = components.into_iter().map(String::from).collect();

            return Ok(parsed);
        }

        // Detect Unix absolute path
        if self.unix_absolute_regex.is_match(path) {
            parsed.is_absolute = true;

            let components: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
            parsed.components = components.into_iter().map(String::from).collect();

            return Ok(parsed);
        }

        // Relative path
        let components: Vec<&str> = path
            .split(['/', '\\'])
            .filter(|s| !s.is_empty())
            .collect();
        parsed.components = components.into_iter().map(String::from).collect();

        Ok(parsed)
    }

    fn parse_unc_path(&self, path: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = path.split('\\').filter(|s| !s.is_empty()).collect();
        if parts.len() >= 2 {
            return Some((parts[0].to_string(), parts[1].to_string()));
        }
        None
    }

    /// Detect path style
    #[must_use] 
    pub fn detect_style(path: &str) -> super::PathStyle {
        let parser = Self::new();

        if parser.unc_path_regex.is_match(path) || parser.windows_absolute_regex.is_match(path) {
            super::PathStyle::Windows
        } else if parser.unix_absolute_regex.is_match(path) {
            super::PathStyle::Unix
        } else if path.contains('\\') && !path.contains('/') {
            super::PathStyle::Windows
        } else if path.contains('/') && !path.contains('\\') {
            super::PathStyle::Unix
        } else {
            super::platform::current_style()
        }
    }

    /// Normalize path by removing redundant components
    pub fn normalize_path(path: &Path) -> PathResult<PathBuf> {
        let mut components = Vec::new();

        for component in path.components() {
            match component {
                std::path::Component::Prefix(_) | std::path::Component::RootDir => {
                    components.clear();
                    components.push(component);
                }
                std::path::Component::CurDir => {
                    // Ignore current directory
                }
                std::path::Component::ParentDir => {
                    if components.is_empty() {
                        components.push(component);
                    } else {
                        let last = components.last().unwrap();
                        match last {
                            std::path::Component::ParentDir
                            | std::path::Component::RootDir
                            | std::path::Component::Prefix(_) => {
                                components.push(component);
                            }
                            _ => {
                                components.pop();
                            }
                        }
                    }
                }
                std::path::Component::Normal(name) => {
                    components.push(std::path::Component::Normal(name));
                }
            }
        }

        let mut normalized = PathBuf::new();
        for component in components {
            normalized.push(component.as_os_str());
        }

        Ok(normalized)
    }
}

/// Parsed path information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedPath {
    /// Original path string
    pub original: String,
    /// Path components
    pub components: Vec<String>,
    /// Whether path is absolute
    pub is_absolute: bool,
    /// Whether path has drive letter
    pub has_drive: bool,
    /// Drive letter (if present)
    pub drive_letter: Option<char>,
    /// Whether path is UNC
    pub is_unc: bool,
    /// UNC server name
    pub server: Option<String>,
    /// UNC share name
    pub share: Option<String>,
}
