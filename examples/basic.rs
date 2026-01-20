use cross_path::{CrossPath, PathConfig, PathConvert, PathStyle};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Basic path conversion
    let windows_path = r"C:\Users\John\Documents\file.txt";
    let unix_path = "/home/john/documents/file.txt";

    // Create cross-platform paths
    let cp1 = CrossPath::new(windows_path)?;
    println!("Original path: {}", cp1.as_original().display());
    println!("To Unix: {}", cp1.to_unix()?);

    let cp2 = CrossPath::new(unix_path)?;
    println!("Original path: {}", cp2.as_original().display());
    println!("To Windows: {}", cp2.to_windows()?);

    // Use custom configuration
    let config = PathConfig {
        style: PathStyle::Auto,
        preserve_encoding: true,
        security_check: true,
        drive_mappings: vec![
            ("C:".to_string(), "/mnt/c".to_string()),
            ("D:".to_string(), "/mnt/data".to_string()),
        ],
        normalize: true,
    };

    let cp3 = CrossPath::with_config(windows_path, config)?;
    println!("Custom mapping: {}", cp3.to_unix()?);

    // Direct string conversion
    let converted = windows_path.to_unix_path()?;
    println!("Direct conversion: {}", converted);

    Ok(())
}
