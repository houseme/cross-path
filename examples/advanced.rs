use cross_path::{CrossPath, security, unicode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Advanced usage: encoding and security handling

    // 1. Encoding handling
    let windows_path_with_unicode = "C:\\Users\\张三\\文档\\文件.txt";
    let cp = CrossPath::new(windows_path_with_unicode)?;

    println!("Unicode path: {}", cp.as_original().display());
    println!(
        "Sanitized path: {}",
        security::PathSecurityChecker::sanitize_path(windows_path_with_unicode)
    );

    // 2. Security checking
    let dangerous_path = "../../etc/passwd";
    match CrossPath::new(dangerous_path) {
        Ok(path) => {
            if let Err(e) = path.is_safe() {
                println!("Security warning: {}", e);
            }
        }
        Err(e) => println!("Path error: {}", e),
    }

    // 3. Batch conversion
    let paths = vec![
        r"C:\Program Files\App\config.ini",
        "/usr/local/bin/app",
        r"\\server\share\file.txt",
        "relative/path/to/file",
    ];

    for path in paths {
        match CrossPath::new(path) {
            Ok(cp) => {
                println!("\nProcessing path: {}", path);
                println!("  To Windows: {:?}", cp.to_windows());
                println!("  To Unix: {:?}", cp.to_unix());
                println!("  Platform appropriate: {:?}", cp.to_platform());
            }
            Err(e) => println!("Error processing {}: {}", path, e),
        }
    }

    // 4. UNC path handling
    let unc_path = r"\\server\share\folder\file.txt";
    let cp_unc = CrossPath::new(unc_path)?;
    println!("\nUNC path conversion:");
    println!("  Unix format: {}", cp_unc.to_unix()?);
    println!("  Windows format: {}", cp_unc.to_windows()?);

    // 5. Encoding detection and conversion
    let bytes = b"C:\\Users\\\x93\x65\x88\x97\\file.txt";
    match unicode::UnicodeHandler::convert_to_utf8(bytes) {
        Ok(utf8_string) => println!("\nDecoded UTF-8: {}", utf8_string),
        Err(e) => println!("\nEncoding error: {}", e),
    }

    Ok(())
}
