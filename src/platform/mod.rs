//! Platform-specific path handling operations
//!
//! This module provides traits and structures for handling platform-specific
//! path operations, file attributes, and disk information.
//!
//! It abstracts away the differences between Windows and Unix-like systems,
//! allowing for uniform access to filesystem metadata.

#[cfg(not(target_os = "windows"))]
pub mod unix;
#[cfg(target_os = "windows")]
pub mod windows;

use alloc::string::String;
use core::option::Option;
#[cfg(not(target_os = "windows"))]
pub use unix::UnixPathExt;
#[cfg(target_os = "windows")]
pub use windows::WindowsPathExt;

use super::PathStyle;

/// Get current platform path style
#[must_use]
pub fn current_style() -> PathStyle {
    #[cfg(target_os = "windows")]
    {
        PathStyle::Windows
    }

    #[cfg(not(target_os = "windows"))]
    {
        PathStyle::Unix
    }
}

/// Platform-specific path operations
pub trait PlatformPath {
    /// Get platform-specific path separator
    fn separator(&self) -> char;

    /// Check if path is absolute
    fn is_absolute(&self) -> bool;

    /// Convert to platform-specific canonical path
    fn to_platform_specific(&self) -> String;
}

/// Platform extension trait
pub trait PathExt: PlatformPath {
    /// Get file attributes (platform-specific)
    fn get_attributes(&self) -> Option<FileAttributes>;

    /// Check if path exists and is accessible
    fn is_accessible(&self) -> bool;

    /// Get disk information for path
    fn get_disk_info(&self) -> Option<DiskInfo>;
}

/// File attributes structure
#[derive(Debug, Clone)]
pub struct FileAttributes {
    /// File size in bytes
    pub size: u64,
    /// Whether the file is a directory
    pub is_directory: bool,
    /// Whether the file is hidden
    pub is_hidden: bool,
    /// Whether the file is read-only
    pub is_readonly: bool,
    /// Creation timestamp (if available)
    pub creation_time: Option<u64>,
    /// Last modification timestamp (if available)
    pub modification_time: Option<u64>,
}

/// Disk information structure
#[derive(Debug, Clone)]
pub struct DiskInfo {
    /// Total disk space in bytes
    pub total_space: u64,
    /// Free disk space in bytes
    pub free_space: u64,
    /// Filesystem type name (e.g., "NTFS", "ext4")
    pub filesystem_type: String,
}
