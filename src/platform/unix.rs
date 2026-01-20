use crate::platform::{DiskInfo, FileAttributes, PathExt, PlatformPath};
use crate::PathError;
use std::fs;
use std::path::Path;

/// Unix platform path extension
pub struct UnixPathExt;

impl PlatformPath for UnixPathExt {
    fn separator(&self) -> char {
        '/'
    }

    fn is_absolute(&self) -> bool {
        true // Unix paths can always be determined as absolute
    }

    fn to_platform_specific(&self) -> String {
        "Unix".to_string()
    }
}

impl PathExt for UnixPathExt {
    fn get_attributes(&self) -> Option<FileAttributes> {
        None // Simplified implementation
    }

    fn is_accessible(&self) -> bool {
        false // Simplified implementation
    }

    fn get_disk_info(&self) -> Option<DiskInfo> {
        None // Simplified implementation
    }
}

/// Check if string is an absolute Unix path
pub fn is_absolute_unix_path(path: &str) -> bool {
    path.starts_with('/')
}

/// Parse Unix mount point from path
pub fn parse_unix_mount_point(path: &str) -> Option<(&str, &str)> {
    if let Some(stripped) = path.strip_prefix("/mnt/") {
        if let Some(pos) = stripped.find('/') {
            let drive = &stripped[..pos];
            let rest = &stripped[pos..];
            return Some((drive, rest));
        }
    }

    if let Some(stripped) = path.strip_prefix('/') {
        if let Some(pos) = stripped.find('/') {
            let first_component = &stripped[..pos];
            if first_component.len() == 1
                && first_component.chars().next().unwrap().is_ascii_lowercase()
            {
                return Some((first_component, &stripped[pos..]));
            }
        }
    }

    None
}

/// Get Unix path statistics
pub fn get_unix_path_stats(path: &Path) -> Result<PathStats, PathError> {
    let metadata = fs::metadata(path)?;

    Ok(PathStats {
        size: metadata.len(),
        is_dir: metadata.is_dir(),
        permissions: metadata.permissions(),
        modified: metadata.modified().ok(),
        accessed: metadata.accessed().ok(),
        created: metadata.created().ok(),
    })
}

/// Unix path statistics structure
#[derive(Debug, Clone)]
pub struct PathStats {
    pub size: u64,
    pub is_dir: bool,
    pub permissions: std::fs::Permissions,
    pub modified: Option<std::time::SystemTime>,
    pub accessed: Option<std::time::SystemTime>,
    pub created: Option<std::time::SystemTime>,
}

/// Check if path is in standard Unix directories
pub fn is_standard_unix_directory(path: &str) -> bool {
    let standard_dirs = vec![
        "/bin", "/boot", "/dev", "/etc", "/home", "/lib", "/lib64", "/media", "/mnt", "/opt",
        "/proc", "/root", "/run", "/sbin", "/srv", "/sys", "/tmp", "/usr", "/var",
    ];

    standard_dirs.iter().any(|&dir| path.starts_with(dir))
}

/// Get filesystem statistics for Unix path
pub fn get_filesystem_stats(path: &Path) -> Result<FilesystemStats, PathError> {
    let path_cstr = std::ffi::CString::new(path.to_string_lossy().as_ref())
        .map_err(|e| PathError::platform_error(e.to_string()))?;

    let mut statfs: libc::statvfs = unsafe { std::mem::zeroed() };

    unsafe {
        if libc::statvfs(path_cstr.as_ptr(), &mut statfs) != 0 {
            return Err(PathError::platform_error(format!(
                "Failed to get filesystem stats for {:?}",
                path
            )));
        }
    }

    Ok(FilesystemStats {
        block_size: statfs.f_bsize as u64,
        total_blocks: statfs.f_blocks,
        free_blocks: statfs.f_bfree,
        available_blocks: statfs.f_bavail,
        total_inodes: statfs.f_files,
        free_inodes: statfs.f_ffree,
        filesystem_id: statfs.f_fsid,
        mount_flags: statfs.f_flag,
        max_filename_length: statfs.f_namemax as u64,
    })
}

/// Filesystem statistics structure
#[derive(Debug, Clone)]
pub struct FilesystemStats {
    pub block_size: u64,
    pub total_blocks: u64,
    pub free_blocks: u64,
    pub available_blocks: u64,
    pub total_inodes: u64,
    pub free_inodes: u64,
    pub filesystem_id: u64,
    pub mount_flags: u64,
    pub max_filename_length: u64,
}
