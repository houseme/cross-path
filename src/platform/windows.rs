use crate::platform::{DiskInfo, FileAttributes, PathExt, PlatformPath};
use crate::PathError;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::iter::Iterator;
use core::option::Option;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::core::PCWSTR;
use windows::Win32::Foundation::GetLastError;
use windows::Win32::Storage::FileSystem::GetFileAttributesW;

/// Windows platform path extension
pub struct WindowsPathExt;

impl PlatformPath for WindowsPathExt {
    fn separator(&self) -> char {
        '\\'
    }

    fn is_absolute(&self) -> bool {
        true // Windows paths can always be determined as absolute
    }

    fn to_platform_specific(&self) -> String {
        "Windows".to_string()
    }
}

impl PathExt for WindowsPathExt {
    fn get_attributes(&self) -> Option<FileAttributes> {
        None // Simplified implementation, actual implementation would call Windows API
    }

    fn is_accessible(&self) -> bool {
        false // Simplified implementation
    }

    fn get_disk_info(&self) -> Option<DiskInfo> {
        None // Simplified implementation
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
        .map_err(|e| PathError::encoding_error(e.to_string()))?;

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
