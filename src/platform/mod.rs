#[cfg(not(target_os = "windows"))]
mod unix;
#[cfg(target_os = "windows")]
mod windows;

use alloc::string::String;
use core::option::Option;
#[cfg(not(target_os = "windows"))]
pub use unix::UnixPathExt;
#[cfg(target_os = "windows")]
pub use windows::WindowsPathExt;

use super::PathStyle;

/// Get current platform path style
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
    pub size: u64,
    pub is_directory: bool,
    pub is_hidden: bool,
    pub is_readonly: bool,
    pub creation_time: Option<u64>,
    pub modification_time: Option<u64>,
}

/// Disk information structure
#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub total_space: u64,
    pub free_space: u64,
    pub filesystem_type: String,
}
