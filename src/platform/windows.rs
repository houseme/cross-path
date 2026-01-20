//! Windows-specific path handling implementation
//!
//! This module provides Windows-specific implementations for path operations,
//! including UTF-16 conversion, drive letter handling, and Windows API integration.
//!
//! It uses the `windows` crate to interact with the Windows API.

use crate::PathError;
use crate::platform::{DiskInfo, FileAttributes, PathExt, PlatformPath};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::iter::Iterator;
use core::option::Option;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
use windows::Win32::Foundation::GetLastError;
use windows::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_HIDDEN, GetDiskFreeSpaceExW, GetFileAttributesW, GetVolumeInformationW,
};
use windows::core::PCWSTR;

/// Windows platform path extension
pub struct WindowsPathExt {
    path: PathBuf,
}

impl WindowsPathExt {
    /// Create new WindowsPathExt
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
}

impl PlatformPath for WindowsPathExt {
    fn separator(&self) -> char {
        '\\'
    }

    fn is_absolute(&self) -> bool {
        self.path.is_absolute()
    }

    fn to_platform_specific(&self) -> String {
        self.path.to_string_lossy().into_owned()
    }
}

impl PathExt for WindowsPathExt {
    fn get_attributes(&self) -> Option<FileAttributes> {
        let metadata = std::fs::metadata(&self.path).ok()?;

        let size = metadata.len();
        let is_directory = metadata.is_dir();
        let is_readonly = metadata.permissions().readonly();

        // Get hidden attribute using Windows metadata
        let attrs = metadata.file_attributes();
        let is_hidden = (attrs & FILE_ATTRIBUTE_HIDDEN.0) != 0;

        let creation_time = metadata
            .created()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());

        let modification_time = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());

        Some(FileAttributes {
            size,
            is_directory,
            is_hidden,
            is_readonly,
            creation_time,
            modification_time,
        })
    }

    fn is_accessible(&self) -> bool {
        self.path.exists()
    }

    fn get_disk_info(&self) -> Option<DiskInfo> {
        // Find root path (e.g., "C:\" or "\\Server\Share\")
        let root = self.path.components().next().and_then(|c| match c {
            std::path::Component::Prefix(prefix) => {
                let mut s = prefix.as_os_str().to_os_string();
                s.push("\\");
                Some(s)
            }
            std::path::Component::RootDir => Some(std::path::PathBuf::from("\\").into_os_string()),
            _ => None,
        })?;

        let root_str = root.to_string_lossy();
        let wide_root = to_windows_path(&root_str).ok()?;

        let mut total_bytes = 0u64;
        let mut free_bytes_caller = 0u64;
        let mut total_free_bytes = 0u64;

        unsafe {
            let result = GetDiskFreeSpaceExW(
                PCWSTR(wide_root.as_ptr()),
                Some(&mut free_bytes_caller),
                Some(&mut total_bytes),
                Some(&mut total_free_bytes),
            );

            if result.is_err() {
                return None;
            }
        }

        // Get Filesystem Name
        let mut fs_name_buf = [0u16; 256];
        let fs_type = unsafe {
            let res = GetVolumeInformationW(
                PCWSTR(wide_root.as_ptr()),
                None,
                None,
                None,
                None,
                Some(&mut fs_name_buf),
            );

            if res.is_ok() {
                let len = fs_name_buf
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(fs_name_buf.len());
                String::from_utf16_lossy(&fs_name_buf[..len])
            } else {
                "Unknown".to_string()
            }
        };

        Some(DiskInfo {
            total_space: total_bytes,
            free_space: free_bytes_caller,
            filesystem_type: fs_type,
        })
    }
}

/// Convert string to Windows UTF-16 path
pub fn to_windows_path(path: &str) -> Result<Vec<u16>, PathError> {
    let mut wide: Vec<u16> = path.encode_utf16().collect();
    wide.push(0); // Add null terminator

    // Convert separators
    for ch in &mut wide {
        if *ch == b'/' as u16 {
            *ch = b'\\' as u16;
        }
    }

    Ok(wide)
}

/// Convert Windows UTF-16 path to string
pub fn from_windows_path(wide: &[u16]) -> Result<String, PathError> {
    // Find null terminator
    let null_pos = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
    let slice = &wide[..null_pos];

    // Convert separators
    let mut result = OsString::from_wide(slice)
        .into_string()
        .map_err(|e| PathError::encoding_error(e.to_string_lossy().into_owned()))?;

    // Unify separator display
    result = result.replace('\\', "/");

    Ok(result)
}

/// Check if string is a valid Windows path
pub fn is_valid_windows_path(path: &str) -> bool {
    // Check drive letter format
    if path.len() >= 2 {
        let first_char = path.chars().next().unwrap();
        let second_char = path.chars().nth(1).unwrap();

        if first_char.is_ascii_alphabetic() && second_char == ':' {
            return true;
        }
    }

    // Check UNC path
    if path.starts_with(r"\\") {
        return true;
    }

    false
}

/// Extract drive letter from Windows path
pub fn get_drive_letter(path: &str) -> Option<char> {
    if path.len() >= 2 {
        let first_char = path.chars().next().unwrap();
        let second_char = path.chars().nth(1).unwrap();

        if first_char.is_ascii_alphabetic() && second_char == ':' {
            return Some(first_char.to_ascii_uppercase());
        }
    }

    None
}

/// Get Windows file attributes using Windows API
pub fn get_windows_file_attributes(path: &str) -> Result<u32, PathError> {
    let wide_path = to_windows_path(path)?;

    unsafe {
        let attrs = GetFileAttributesW(PCWSTR(wide_path.as_ptr()));

        if attrs == 0xFFFFFFFF {
            let error = GetLastError();
            return Err(PathError::platform_error(format!(
                "Failed to get file attributes: {:?}",
                error
            )));
        }

        Ok(attrs)
    }
}

/// Check if path exists on Windows
pub fn windows_path_exists(path: &str) -> Result<bool, PathError> {
    let attrs = get_windows_file_attributes(path)?;
    Ok(attrs != 0xFFFFFFFF)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_windows_path() {
        assert!(is_valid_windows_path(r"C:\Windows"));
        assert!(is_valid_windows_path(r"D:\Data\file.txt"));
        assert!(is_valid_windows_path(r"\\Server\Share"));
        assert!(!is_valid_windows_path(r"/usr/bin"));
        assert!(!is_valid_windows_path(r"relative\path"));
    }

    #[test]
    fn test_get_drive_letter() {
        assert_eq!(get_drive_letter(r"C:\Windows"), Some('C'));
        assert_eq!(get_drive_letter(r"d:\data"), Some('D'));
        assert_eq!(get_drive_letter(r"\\Server\Share"), None);
        assert_eq!(get_drive_letter(r"/usr/bin"), None);
    }

    #[test]
    fn test_to_windows_path() {
        let path = "C:/Windows/System32";
        let wide = to_windows_path(path).unwrap();

        // Check null terminator
        assert_eq!(*wide.last().unwrap(), 0);

        // Check separator conversion
        let backslash = b'\\' as u16;
        assert!(wide.contains(&backslash));
    }

    #[test]
    fn test_from_windows_path() {
        let wide = vec![
            'C' as u16,
            ':' as u16,
            '\\' as u16,
            'T' as u16,
            'e' as u16,
            's' as u16,
            't' as u16,
            0,
        ];
        let path = from_windows_path(&wide).unwrap();

        // Should be normalized to forward slashes by default in this lib
        assert_eq!(path, "C:/Test");
    }
}
