# CrossPath

高级跨平台路径处理库，完美支持 Windows 和 Linux 路径互转。

## 特性

- ✅ Windows ↔ Linux 路径双向转换
- ✅ UNC 路径支持
- ✅ 编码自动检测和转换（UTF-8、UTF-16、Windows-1252）
- ✅ 路径安全性检查（防止路径遍历攻击）
- ✅ 可配置的盘符映射
- ✅ 路径规范化
- ✅ 零成本抽象，高性能
- ✅ 完整的错误处理
- ✅ 支持 Serde 序列化

## 安装

```toml
[dependencies]
cross_path = "0.1"
```

## 快速开始

```rust
use cross_path::CrossPath;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Windows 转 Unix
    let cp = CrossPath::new(r"C:\Users\John\file.txt")?;
    println!("Unix 路径：{}", cp.to_unix()?); // /mnt/c/Users/John/file.txt

    // Unix 转 Windows
    let cp = CrossPath::new("/home/john/file.txt")?;
    println!("Windows 路径：{}", cp.to_windows()?); // C:\home\john\file.txt

    // 直接转换
    let unix_path = r"C:\Users\test".to_unix_path()?;
    println!("直接转换：{}", unix_path);

    Ok(())
}
```

## 高级用法

### 自定义配置

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
println!("转换后：{}", cp.to_unix()?); // /mnt/data/Data/file.txt
```

### 安全性检查

```rust
use cross_path::security::PathSecurityChecker;

let checker = PathSecurityChecker::new();
let path = std::path::Path::new("../../etc/passwd");

match checker.check(path) {
Ok(_) => println ! ("路径安全"),
Err(e) => println ! ("安全警告：{}", e),
}

// 清理危险路径
let safe_path = PathSecurityChecker::sanitize_path("file<>.txt");
println!("安全路径：{}", safe_path); // file__.txt
```

### 编码处理

```rust
use cross_path::unicode::UnicodeHandler;

// 检测编码
let bytes = b"C:\\Users\\\x93\x65\x88\x97\\file.txt";
let encoding = UnicodeHandler::detect_encoding(bytes);
println!("检测到的编码：{}", encoding.name());

// 转换到 UTF-8
let utf8_string = UnicodeHandler::to_utf8(bytes) ?;
println!("UTF-8 字符串：{}", utf8_string);
```

## API 文档

运行以下命令查看详细 API 文档：

```bash
cargo doc --open
```

## 支持的功能

### 路径转换

- [x] Windows绝对路径 ↔ Unix绝对路径
- [x] 相对路径转换
- [x] UNC 路径转换
- [x] 盘符映射
- [x] 分隔符统一

### 安全性

- [x] 路径遍历攻击检测
- [x] 危险模式检测
- [x] Windows 保留名称检查
- [x] 系统目录访问控制

### 编码支持

- [x] UTF-8
- [x] UTF-16 LE
- [x] Windows-1252
- [x] 编码自动检测

## 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test --test conversion

# 运行性能测试
cargo bench
```

## 贡献指南

1. Fork 仓库
2. 创建特性分支
3. 提交更改
4. 推送分支
5. 创建 Pull Request

## 许可证

MIT OR Apache-2.0