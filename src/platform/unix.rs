//! Unix-specific path handling implementation
//!
//! This module provides Unix-specific implementations for path operations,
//! including file attribute retrieval, filesystem statistics, and path normalization.
//!
//! It uses POSIX standard APIs (via `libc`) to interact with the underlying system.

use crate::PathError;
use crate::platform::{DiskInfo, FileAttributes, PathExt, PlatformPath};
use std::fs;
use std::path::{Path, PathBuf};

/// Unix platform path extension
pub struct UnixPathExt {
    path: PathBuf,
}

impl UnixPathExt {
    /// Create new `UnixPathExt`
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
}

impl PlatformPath for UnixPathExt {
    fn separator(&self) -> char {
        '/'
    }

    fn is_absolute(&self) -> bool {
        self.path.is_absolute()
    }

    fn to_platform_specific(&self) -> String {
        self.path.to_string_lossy().into_owned()
    }
}

impl PathExt for UnixPathExt {
    fn get_attributes(&self) -> Option<FileAttributes> {
        let metadata = fs::metadata(&self.path).ok()?;

        let size = metadata.len();
        let is_directory = metadata.is_dir();
        let is_readonly = metadata.permissions().readonly();

        // Unix hidden file convention: starts with '.'
        let is_hidden = self
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|s| s.starts_with('.'));

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
        let stats = get_filesystem_stats(&self.path).ok()?;

        Some(DiskInfo {
            total_space: stats.total_blocks.saturating_mul(stats.block_size),
            free_space: stats.available_blocks.saturating_mul(stats.block_size),
            filesystem_type: "Unix".to_string(),
        })
    }
}

/// Check if string is an absolute Unix path
#[must_use]
pub fn is_absolute_unix_path(path: &str) -> bool {
    path.starts_with('/')
}

/// Parse Unix mount point from path
///
/// # Panics
///
/// Panics if the path contains invalid characters or structure that cannot be parsed.
#[must_use]
pub fn parse_unix_mount_point(path: &str) -> Option<(&str, &str)> {
    if let Some(stripped) = path.strip_prefix("/mnt/")
        && let Some(pos) = stripped.find('/')
    {
        let drive = &stripped[..pos];
        let rest = &stripped[pos..];
        return Some((drive, rest));
    }

    if let Some(stripped) = path.strip_prefix('/')
        && let Some(pos) = stripped.find('/')
    {
        let first_component = &stripped[..pos];
        if first_component.len() == 1
            && first_component.chars().next().unwrap().is_ascii_lowercase()
        {
            return Some((first_component, &stripped[pos..]));
        }
    }

    None
}

/// Get Unix path statistics
///
/// # Arguments
/// * `path` - Path to retrieve statistics for
///
/// # Errors
/// Returns `PathError` if unable to retrieve metadata
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
    /// File size in bytes
    pub size: u64,
    /// Whether the path is a directory
    pub is_dir: bool,
    /// File permissions
    pub permissions: fs::Permissions,
    /// Last modification time
    pub modified: Option<std::time::SystemTime>,
    /// Last access time
    pub accessed: Option<std::time::SystemTime>,
    /// Creation time
    pub created: Option<std::time::SystemTime>,
}

/// Check if path is in standard Unix directories
#[must_use]
pub fn is_standard_unix_directory(path: &str) -> bool {
    let standard_dirs = vec![
        "/bin", "/boot", "/dev", "/etc", "/home", "/lib", "/lib64", "/media", "/mnt", "/opt",
        "/proc", "/root", "/run", "/sbin", "/srv", "/sys", "/tmp", "/usr", "/var",
    ];

    standard_dirs.iter().any(|&dir| path.starts_with(dir))
}

/// Get filesystem statistics for Unix path
///
/// # Errors
///
/// Returns `PathError` if the filesystem statistics cannot be retrieved.
pub fn get_filesystem_stats(path: &Path) -> Result<FilesystemStats, PathError> {
    let path_cstr = std::ffi::CString::new(path.to_string_lossy().as_ref())
        .map_err(|e| PathError::platform_error(e.to_string()))?;

    let mut statfs: libc::statvfs = unsafe { std::mem::zeroed() };

    unsafe {
        if libc::statvfs(path_cstr.as_ptr(), &raw mut statfs) != 0 {
            return Err(PathError::platform_error(format!(
                "Failed to get filesystem stats for {}",
                path.display()
            )));
        }
    }

    Ok(FilesystemStats {
        block_size: statfs.f_bsize as u64,
        total_blocks: u64::from(statfs.f_blocks),
        free_blocks: u64::from(statfs.f_bfree),
        available_blocks: u64::from(statfs.f_bavail),
        total_inodes: u64::from(statfs.f_files),
        free_inodes: u64::from(statfs.f_ffree),
        filesystem_id: statfs.f_fsid as u64,
        mount_flags: statfs.f_flag as u64,
        max_filename_length: statfs.f_namemax as u64,
    })
}

/// Filesystem statistics structure
#[derive(Debug, Clone)]
pub struct FilesystemStats {
    /// Filesystem block size
    pub block_size: u64,
    /// Total number of blocks
    pub total_blocks: u64,
    /// Number of free blocks
    pub free_blocks: u64,
    /// Number of available blocks for unprivileged users
    pub available_blocks: u64,
    /// Total number of inodes
    pub total_inodes: u64,
    /// Number of free inodes
    pub free_inodes: u64,
    /// Filesystem ID
    pub filesystem_id: u64,
    /// Mount flags
    pub mount_flags: u64,
    /// Maximum filename length
    pub max_filename_length: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_is_absolute_unix_path() {
        assert!(is_absolute_unix_path("/home/user"));
        assert!(is_absolute_unix_path("/"));
        assert!(!is_absolute_unix_path("relative/path"));
        assert!(!is_absolute_unix_path("./relative"));
    }

    #[test]
    fn test_parse_unix_mount_point() {
        assert_eq!(
            parse_unix_mount_point("/mnt/c/Users"),
            Some(("c", "/Users"))
        );
        assert_eq!(parse_unix_mount_point("/c/Users"), Some(("c", "/Users")));
        assert_eq!(parse_unix_mount_point("/home/user"), None);
    }

    #[test]
    fn test_is_standard_unix_directory() {
        assert!(is_standard_unix_directory("/bin/bash"));
        assert!(is_standard_unix_directory("/usr/local/bin"));
        assert!(!is_standard_unix_directory("/my/custom/path"));
    }

    #[test]
    fn test_unix_path_ext() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");
        File::create(&file_path).unwrap();

        let ext = UnixPathExt::new(&file_path);

        assert_eq!(ext.separator(), '/');
        assert!(ext.is_absolute());
        assert!(ext.is_accessible());

        let attrs = ext.get_attributes().unwrap();
        assert!(!attrs.is_directory);
        assert!(!attrs.is_hidden);
        assert_eq!(attrs.size, 0);

        // Test hidden file
        let hidden_path = temp_dir.path().join(".hidden");
        File::create(&hidden_path).unwrap();
        let hidden_ext = UnixPathExt::new(&hidden_path);
        let hidden_attrs = hidden_ext.get_attributes().unwrap();
        assert!(hidden_attrs.is_hidden);
    }
}
