use cross_path::{CrossPath, security, unicode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Advanced Path Handling Examples ===\n");

    // 1. Encoding handling
    println!("1. Unicode Path Handling:");
    let windows_path_with_unicode = "C:\\Users\\张三\\文档\\文件.txt";
    let cp = CrossPath::new(windows_path_with_unicode)?;

    println!("   Original: {}", cp.as_original().display());
    println!(
        "   Sanitized: {}",
        security::PathSecurityChecker::sanitize_path(windows_path_with_unicode)
    );
    println!();

    // 2. Security checking
    println!("2. Security Checking:");
    let dangerous_path = "../../etc/passwd";
    println!("   Checking path: {}", dangerous_path);
    match CrossPath::new(dangerous_path) {
        Ok(path) => match path.is_safe() {
            Ok(safe) => println!("   Is safe? {}", safe),
            Err(e) => println!("   Security warning: {}", e),
        },
        Err(e) => println!("   Path error: {}", e),
    }
    println!();

    // 3. Batch conversion
    println!("3. Batch Processing:");
    let paths = vec![
        r"C:\Program Files\App\config.ini",
        "/usr/local/bin/app",
        r"\\server\share\file.txt",
        "relative/path/to/file",
    ];

    for path in paths {
        match CrossPath::new(path) {
            Ok(cp) => {
                println!("   Path: {}", path);
                println!("     -> Windows: {:?}", cp.to_windows().unwrap_or_default());
                println!("     -> Unix:    {:?}", cp.to_unix().unwrap_or_default());
                println!(
                    "     -> Native:  {:?}",
                    cp.to_platform().unwrap_or_default()
                );
            }
            Err(e) => println!("   Error processing {}: {}", path, e),
        }
    }
    println!();

    // 4. UNC path handling
    println!("4. UNC Path Handling:");
    let unc_path = r"\\server\share\folder\file.txt";
    let cp_unc = CrossPath::new(unc_path)?;
    println!("   Original: {}", unc_path);
    println!("   Unix format: {}", cp_unc.to_unix()?);
    println!("   Windows format: {}", cp_unc.to_windows()?);
    println!();

    // 5. Encoding detection and conversion
    #[cfg(feature = "unicode")]
    {
        println!("5. Encoding Detection (GBK/GB18030 simulation):");
        // Simulating some non-UTF8 bytes (e.g. GBK encoded "中文")
        // Note: This is just a demonstration, actual byte sequences depend on encoding
        let bytes = b"C:\\Users\\\xd6\xd0\xce\xc4\\file.txt";
        match unicode::UnicodeHandler::convert_to_utf8(bytes) {
            Ok(utf8_string) => println!("   Decoded UTF-8: {}", utf8_string),
            Err(e) => println!("   Encoding error: {}", e),
        }
    }

    Ok(())
}
