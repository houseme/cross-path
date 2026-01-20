use cross_path::CrossPath;

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
