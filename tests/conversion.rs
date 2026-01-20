use cross_path::{CrossPath, PathConfig};

#[test]
fn test_windows_to_unix_conversion() {
    let windows_path = r"C:\Users\test\file.txt";
    let cp = CrossPath::new(windows_path).unwrap();

    assert_eq!(cp.to_unix().unwrap(), "/mnt/c/Users/test/file.txt");
}

#[test]
fn test_unix_to_windows_conversion() {
    let unix_path = "/mnt/c/Users/test/file.txt";
    let cp = CrossPath::new(unix_path).unwrap();

    assert_eq!(cp.to_windows().unwrap(), r"C:\Users\test\file.txt");
}

#[test]
fn test_relative_path_conversion() {
    let path = "foo/bar/baz";
    let cp = CrossPath::new(path).unwrap();

    // On Unix, it should remain the same (normalized separators)
    assert_eq!(cp.to_unix().unwrap(), "foo/bar/baz");

    // On Windows, it should use backslashes
    assert_eq!(cp.to_windows().unwrap(), r"foo\bar\baz");
}

#[test]
fn test_mixed_separator_conversion() {
    let mixed_path = "foo/bar\\baz";
    let cp = CrossPath::new(mixed_path).unwrap();

    assert_eq!(cp.to_unix().unwrap(), "foo/bar/baz");
    assert_eq!(cp.to_windows().unwrap(), r"foo\bar\baz");
}

#[test]
fn test_drive_letter_case_insensitivity() {
    let lower_drive = r"c:\Users\test";
    let cp = CrossPath::new(lower_drive).unwrap();

    assert_eq!(cp.to_unix().unwrap(), "/mnt/c/Users/test");
}

#[test]
fn test_unc_path_conversion() {
    // Windows UNC to Unix
    let unc_win = r"\\server\share\path\to\file";
    let cp = CrossPath::new(unc_win).unwrap();
    assert_eq!(cp.to_unix().unwrap(), "//server/share/path/to/file");

    // Unix UNC-like to Windows
    let unc_unix = "//server/share/path/to/file";
    let cp2 = CrossPath::new(unc_unix).unwrap();
    assert_eq!(cp2.to_windows().unwrap(), r"\\server\share\path\to\file");
}

#[test]
fn test_root_path_conversion() {
    // Windows Root
    let win_root = r"C:\";
    let cp = CrossPath::new(win_root).unwrap();
    // Expect trailing slash because input had it and it's a root path
    assert_eq!(cp.to_unix().unwrap(), "/mnt/c/");

    // Unix Root
    let unix_root = "/";
    let cp2 = CrossPath::new(unix_root).unwrap();
    assert_eq!(cp2.to_windows().unwrap(), r"C:\");
}

#[test]
fn test_different_drives() {
    let d_drive = r"D:\Data";
    let cp = CrossPath::new(d_drive).unwrap();
    assert_eq!(cp.to_unix().unwrap(), "/mnt/d/Data");

    let e_drive = r"E:\Backup\Image.iso";
    let cp2 = CrossPath::new(e_drive).unwrap();
    assert_eq!(cp2.to_unix().unwrap(), "/mnt/e/Backup/Image.iso");
}

#[test]
fn test_non_standard_unix_path() {
    // Path not in /mnt/
    let unix_path = "/var/log/syslog";
    let cp = CrossPath::new(unix_path).unwrap();
    // Default behavior maps absolute unix paths to C:
    assert_eq!(cp.to_windows().unwrap(), r"C:\var\log\syslog");
}

#[test]
fn test_paths_with_spaces() {
    let path = r"C:\Program Files\App Name\config.ini";
    let cp = CrossPath::new(path).unwrap();
    assert_eq!(
        cp.to_unix().unwrap(),
        "/mnt/c/Program Files/App Name/config.ini"
    );
}

#[test]
fn test_multiple_separators() {
    let path = "foo//bar\\\\baz";
    let cp = CrossPath::new(path).unwrap();
    assert_eq!(cp.to_unix().unwrap(), "foo/bar/baz");
    assert_eq!(cp.to_windows().unwrap(), r"foo\bar\baz");
}

#[test]
fn test_normalization() {
    let path = "foo/./bar/../baz";
    let mut cp = CrossPath::new(path).unwrap();

    // Explicit normalization required to resolve . and ..
    cp.normalize().unwrap();

    assert_eq!(cp.to_unix().unwrap(), "foo/baz");
    assert_eq!(cp.to_windows().unwrap(), r"foo\baz");
}

#[test]
fn test_custom_configuration() {
    let config = PathConfig {
        drive_mappings: vec![("Z:".to_string(), "/network".to_string())],
        ..PathConfig::default()
    };

    let path = r"Z:\shared\doc.txt";
    let cp = CrossPath::with_config(path, config).unwrap();

    assert_eq!(cp.to_unix().unwrap(), "/network/shared/doc.txt");
}

#[test]
fn test_custom_configuration_reverse() {
    let config = PathConfig {
        drive_mappings: vec![("Z:".to_string(), "/network".to_string())],
        ..PathConfig::default()
    };

    let path = "/network/shared/doc.txt";
    let cp = CrossPath::with_config(path, config).unwrap();

    assert_eq!(cp.to_windows().unwrap(), r"Z:\shared\doc.txt");
}
