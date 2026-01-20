# CrossPath

[![Build](https://github.com/houseme/cross-path/workflows/Build/badge.svg)](https://github.com/houseme/cross-path/actions?query=workflow%3ABuild)
[![crates.io](https://img.shields.io/crates/v/cross-path.svg)](https://crates.io/crates/cross-path)
[![docs.rs](https://docs.rs/cross-path/badge.svg)](https://docs.rs/cross-path/)
[![License](https://img.shields.io/crates/l/cross-path)](./LICENSE-APACHE)
[![Downloads](https://img.shields.io/crates/d/cross-path)](https://crates.io/crates/cross-path)

English | [简体中文](README_ZH.md)

Advanced cross-platform path handling library for perfect Windows and Unix path conversion.

## Features

- ✅ Windows ↔ Unix bidirectional path conversion
- ✅ UNC path support
- ✅ Automatic encoding detection and conversion (UTF-8, UTF-16, Windows-1252)
- ✅ Path security verification (path traversal prevention)
- ✅ Configurable drive letter mappings
- ✅ Path normalization
- ✅ Zero-cost abstractions, high performance
- ✅ Comprehensive error handling
- ✅ Serde serialization support

## Installation

```toml
[dependencies]
cross_path = "0.0.1"
```

## Quick Start

```rust
use cross_path::CrossPath;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Windows to Unix conversion
    let cp = CrossPath::new(r"C:\Users\John\file.txt")?;
    println!("Unix path: {}", cp.to_unix()?); // /mnt/c/Users/John/file.txt

    // Unix to Windows conversion
    let cp = CrossPath::new("/home/john/file.txt")?;
    println!("Windows path: {}", cp.to_windows()?); // C:\home\john\file.txt

    // Direct conversion
    let unix_path = r"C:\Users\test".to_unix_path()?;
    println!("Direct conversion: {}", unix_path);

    Ok(())
}
```

## Advanced Usage

### Custom Configuration

```rust
use cross_path::{CrossPath, PathConfig, PathStyle};

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

let cp = CrossPath::with_config(r"D:\Data\file.txt", config) ?;
println!("Converted: {}", cp.to_unix()?); // /mnt/data/Data/file.txt
```

### Security Checking

```rust
use cross_path::security::PathSecurityChecker;

let checker = PathSecurityChecker::new();
let path = std::path::Path::new("../../etc/passwd");

match checker.check(path) {
Ok(_) => println ! ("Path is safe"),
Err(e) => println ! ("Security warning: {}", e),
}

// Sanitize dangerous paths
let safe_path = PathSecurityChecker::sanitize_path("file<>.txt");
println!("Safe path: {}", safe_path); // file__.txt
```

### Encoding Handling

```rust
use cross_path::unicode::UnicodeHandler;

// Detect encoding
let bytes = b"C:\\Users\\\x93\x65\x88\x97\\file.txt";
let encoding = UnicodeHandler::detect_encoding(bytes);
println!("Detected encoding: {}", encoding.name());

// Convert to UTF-8
let utf8_string = UnicodeHandler::convert_to_utf8(bytes) ?;
println!("UTF-8 string: {}", utf8_string);
```

## API Documentation

For detailed API documentation, run:

```bash
cargo doc --open
```

## Supported Features

### Path Conversion

- [x] Windows absolute path ↔ Unix absolute path
- [x] Relative path conversion
- [x] UNC path conversion
- [x] Drive letter mapping
- [x] Separator unification

### Security

- [x] Path traversal attack detection
- [x] Dangerous pattern detection
- [x] Windows reserved name checking
- [x] System directory access control

### Encoding Support

- [x] UTF-8
- [x] UTF-16 LE
- [x] Windows-1252
- [x] Automatic encoding detection

## Testing

```bash
# Run all tests
cargo test

# Run specific tests
cargo test --test conversion

# Run benchmarks
cargo bench
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push the branch
5. Create a Pull Request

## License

MIT OR Apache-2.0
