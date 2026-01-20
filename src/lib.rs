//! Advanced cross-platform path handling library
//!
//! Provides perfect compatibility handling for Windows and Linux paths, supporting:
//! - Windows ↔ Linux bidirectional path conversion
//! - Automatic encoding detection and conversion
//! - Path security verification
//! - Cross-platform file operations

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
extern crate alloc;

pub mod converter;
pub mod error;
pub mod formatter;
pub mod parser;
pub mod platform;
#[cfg(feature = "security")]
pub mod security;
#[cfg(feature = "unicode")]
pub mod unicode;

pub use converter::PathConverter;
pub use error::PathError;
pub use formatter::PathFormatter;
pub use parser::PathParser;

use std::path::{Path, PathBuf};

/// Cross-platform path result type
pub type PathResult<T> = Result<T, PathError>;

/// Path style enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PathStyle {
    /// Windows path style (C:\Users\name)
    Windows,
    /// Unix/Linux path style (/home/name)
    Unix,
    /// Auto-detect based on current platform
    Auto,
}

/// Path conversion configuration
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PathConfig {
    /// Target path style
    pub style: PathStyle,
    /// Whether to preserve original encoding
    pub preserve_encoding: bool,
    /// Whether to perform security checks
    pub security_check: bool,
    /// Windows drive letter mappings (e.g., "C:" -> "/mnt/c")
    pub drive_mappings: Vec<(String, String)>,
    /// Whether to normalize paths (remove redundant components)
    pub normalize: bool,
}

impl Default for PathConfig {
    fn default() -> Self {
        Self {
            style: PathStyle::Auto,
            preserve_encoding: true,
            security_check: true,
            drive_mappings: default_drive_mappings(),
            normalize: true,
        }
    }
}

/// Default drive letter mappings
fn default_drive_mappings() -> Vec<(String, String)> {
    vec![
        ("C:".to_string(), "/mnt/c".to_string()),
        ("D:".to_string(), "/mnt/d".to_string()),
        ("E:".to_string(), "/mnt/e".to_string()),
    ]
}

/// Main cross-platform path structure
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CrossPath {
    inner: PathBuf,
    original_style: PathStyle,
    config: PathConfig,
}

impl CrossPath {
    /// Create a cross-platform path from a string
    pub fn new<P: AsRef<str>>(path: P) -> PathResult<Self> {
        let path_str = path.as_ref();
        let _ = PathParser::parse(path_str)?;
        let style = PathParser::detect_style(path_str);

        Ok(Self {
            inner: PathBuf::from(path_str),
            original_style: style,
            config: PathConfig::default(),
        })
    }

    /// Create path with custom configuration
    pub fn with_config<P: AsRef<str>>(path: P, config: PathConfig) -> PathResult<Self> {
        let mut cross_path = Self::new(path)?;
        cross_path.config = config;
        Ok(cross_path)
    }

    /// Convert to path string with specified style
    pub fn to_style(&self, style: PathStyle) -> PathResult<String> {
        let converter = PathConverter::new(&self.config);
        converter.convert(self.inner.to_string_lossy().as_ref(), style)
    }

    /// Convert to platform-appropriate path
    pub fn to_platform(&self) -> PathResult<String> {
        let target_style = match self.config.style {
            PathStyle::Auto => platform::current_style(),
            style => style,
        };
        self.to_style(target_style)
    }

    /// Convert to Windows path
    pub fn to_windows(&self) -> PathResult<String> {
        self.to_style(PathStyle::Windows)
    }

    /// Convert to Unix path
    pub fn to_unix(&self) -> PathResult<String> {
        self.to_style(PathStyle::Unix)
    }

    /// Get original path
    pub fn as_original(&self) -> &Path {
        &self.inner
    }

    /// Update configuration
    pub fn set_config(&mut self, config: PathConfig) {
        self.config = config;
    }

    /// Get configuration reference
    pub fn config(&self) -> &PathConfig {
        &self.config
    }

    /// Check if path is safe
    pub fn is_safe(&self) -> PathResult<bool> {
        security::PathSecurityChecker::check_path_security(&self.inner)
    }

    /// Normalize path
    pub fn normalize(&mut self) -> PathResult<()> {
        let normalized = PathParser::normalize_path(&self.inner)?;
        self.inner = normalized;
        Ok(())
    }
}

impl From<&Path> for CrossPath {
    fn from(path: &Path) -> Self {
        Self {
            inner: path.to_path_buf(),
            original_style: PathStyle::Auto,
            config: PathConfig::default(),
        }
    }
}

impl From<PathBuf> for CrossPath {
    fn from(path: PathBuf) -> Self {
        Self {
            inner: path,
            original_style: PathStyle::Auto,
            config: PathConfig::default(),
        }
    }
}

/// Path conversion trait
pub trait PathConvert {
    /// Convert to CrossPath
    fn to_cross_path(&self) -> PathResult<CrossPath>;

    /// Convert to Windows path
    fn to_windows_path(&self) -> PathResult<String>;

    /// Convert to Unix path
    fn to_unix_path(&self) -> PathResult<String>;
}

impl PathConvert for str {
    fn to_cross_path(&self) -> PathResult<CrossPath> {
        CrossPath::new(self)
    }

    fn to_windows_path(&self) -> PathResult<String> {
        let cross_path = CrossPath::new(self)?;
        cross_path.to_windows()
    }

    fn to_unix_path(&self) -> PathResult<String> {
        let cross_path = CrossPath::new(self)?;
        cross_path.to_unix()
    }
}

impl PathConvert for Path {
    fn to_cross_path(&self) -> PathResult<CrossPath> {
        Ok(CrossPath::from(self))
    }

    fn to_windows_path(&self) -> PathResult<String> {
        let cross_path = CrossPath::from(self);
        cross_path.to_windows()
    }

    fn to_unix_path(&self) -> PathResult<String> {
        let cross_path = CrossPath::from(self);
        cross_path.to_unix()
    }
}
