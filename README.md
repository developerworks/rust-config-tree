# rust-config-tree workspace

This workspace contains the **rust-config-tree** family of crates: a Rust library
for recursive include tree utilities for layered configuration files.

### Crates

| Crate                     | Description                                                     | Path                       |
| ------------------------- | --------------------------------------------------------------- | -------------------------- |
| [rust-config-tree]        | Recursive include tree, CLI helpers, template/schema generation | `rust-config-tree/`        |
| [rust-config-tree-macros] | Procedural macros (`ConfigOverrides`, `ConfigSchema`)           | `rust-config-tree-macros/` |

### Quick links

- [📖 Manual](https://developerworks.github.io/rust-config-tree/) — English and 10 translated editions
- [📦 rust-config-tree on crates.io](https://crates.io/crates/rust-config-tree)
- [📦 rust-config-tree-macros on crates.io](https://crates.io/crates/rust-config-tree-macros)
- [📘 API docs](https://docs.rs/rust-config-tree)

### Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
rust-config-tree = "0.2"
```

For the derive macros:

```toml
[dependencies]
rust-config-tree = { version = "0.2", features = ["macros"] }
```

### Development

Build from source:

```bash
cargo build
```

Run tests:

```bash
cargo test
```

Publish a crate (from workspace root):

```bash
cargo publish -p rust-config-tree
cargo publish -p rust-config-tree-macros
```

### License

This project is licensed under [MIT OR Apache-2.0].

[rust-config-tree]: https://crates.io/crates/rust-config-tree
[rust-config-tree-macros]: https://crates.io/crates/rust-config-tree-macros
[MIT OR Apache-2.0]: https://github.com/developerworks/rust-config-tree#license
