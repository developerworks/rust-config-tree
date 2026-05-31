# rust-config-tree workspace

本 workspace（工作区）包含 **rust-config-tree** 系列 crate（包），
一个为分层配置文件提供递归包含树工具的 Rust 库。

### Crate（包）

| Crate                     | 说明                                        | 路径                       |
| ------------------------- | ------------------------------------------- | -------------------------- |
| [rust-config-tree]        | 递归包含树、CLI 辅助、模板/Schema 生成      | `rust-config-tree/`        |
| [rust-config-tree-macros] | 过程宏（`ConfigOverrides`、`ConfigSchema`） | `rust-config-tree-macros/` |

### 快速链接

- [📖 手册](https://developerworks.github.io/rust-config-tree/) — 英文及 10 种语言译本
- [📦 rust-config-tree on crates.io](https://crates.io/crates/rust-config-tree)
- [📦 rust-config-tree-macros on crates.io](https://crates.io/crates/rust-config-tree-macros)
- [📘 API 文档](https://docs.rs/rust-config-tree)

### 使用方式

在 `Cargo.toml` 中添加：

```toml
[dependencies]
rust-config-tree = "0.2"
```

如需派生宏：

```toml
[dependencies]
rust-config-tree = { version = "0.2", features = ["macros"] }
```

### 开发

从源码构建：

```bash
cargo build
```

运行测试：

```bash
cargo test
```

发布 crate（在 workspace 根目录执行）：

```bash
cargo publish -p rust-config-tree
cargo publish -p rust-config-tree-macros
```

### 许可证

本项目使用 [MIT OR Apache-2.0] 许可证。

[rust-config-tree]: https://crates.io/crates/rust-config-tree
[rust-config-tree-macros]: https://crates.io/crates/rust-config-tree-macros
[MIT OR Apache-2.0]: https://github.com/developerworks/rust-config-tree#license
