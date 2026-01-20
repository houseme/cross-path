use cross_path::{CrossPath, PathConfig, PathConvert, PathStyle};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic Path Conversion Examples ===\n");

    // Basic path conversion
    let windows_path = r"C:\Users\John\Documents\file.txt";
    let unix_path = "/home/john/documents/file.txt";

    // Create cross-platform paths
    let cp1 = CrossPath::new(windows_path)?;
    println!("1. Windows to Unix:");
    println!("   Original: {}", cp1.as_original().display());
    println!("   To Unix:  {}", cp1.to_unix()?);
    println!();

    let cp2 = CrossPath::new(unix_path)?;
    println!("2. Unix to Windows:");
    println!("   Original: {}", cp2.as_original().display());
    println!("   To Windows: {}", cp2.to_windows()?);
    println!();

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
    println!("3. Custom Configuration (C: -> /mnt/c, D: -> /mnt/data):");
    println!("   Input: {}", windows_path);
    println!("   Output: {}", cp3.to_unix()?);
    println!();

    // Direct string conversion trait usage
    println!("4. Direct Trait Usage:");
    let converted = windows_path.to_unix_path()?;
    println!("   \"{}\".to_unix_path() -> {}", windows_path, converted);

    Ok(())
}
