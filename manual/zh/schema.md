# 配置结构

[English](../en/schema.html) | [中文](schema.html) | [日本語](../ja/schema.html) | [한국어](../ko/schema.html) | [Français](../fr/schema.html) | [Deutsch](../de/schema.html) | [Español](../es/schema.html) | [Português](../pt/schema.html) | [Svenska](../sv/schema.html) | [Suomi](../fi/schema.html) | [Nederlands](../nl/schema.html)

应用 schema 是普通的 `confique` config 类型。root schema 必须实现
`ConfigSchema`，这样 `rust-config-tree` 才能从中间 `confique` layer 发现
递归 include。

```rust
use std::path::PathBuf;

use confique::Config;
use rust_config_tree::ConfigSchema;

#[derive(Debug, Config)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(nested)]
    database: DatabaseConfig,
}

#[derive(Debug, Config)]
struct DatabaseConfig {
    #[config(env = "APP_DATABASE_URL")]
    url: String,

    #[config(default = 16)]
    #[config(env = "APP_DATABASE_POOL_SIZE")]
    pool_size: u32,
}

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}
```

## Include 字段

include 字段可以使用任意名称。`rust-config-tree` 只通过
`ConfigSchema::include_paths` 了解它。

这个字段通常应有空默认值：

```rust
#[config(default = [])]
include: Vec<PathBuf>,
```

加载器会接收每个文件的部分 layer。这样它可以在最终 schema 合并和校验
之前发现子配置文件。

## 嵌套 Section

使用 `#[config(nested)]` 表示结构化 section。嵌套 section 同时影响运行时
加载和模板拆分：

```rust
#[derive(Debug, Config)]
struct AppConfig {
    #[config(nested)]
    server: ServerConfig,
}
```

对应的自然 YAML 形状为：

```yaml
server:
  bind: 127.0.0.1
  port: 8080
```

## 模板 Section 路径覆盖

当模板 source 没有 include 时，crate 可以从嵌套 schema section 推导子模板
文件。默认顶层路径是 `config/<section>.yaml`。

使用 `template_path_for_section` 覆盖路径：

```rust
impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }

    fn template_path_for_section(section_path: &[&str]) -> Option<PathBuf> {
        match section_path {
            ["database"] => Some(PathBuf::from("examples/database.yaml")),
            _ => None,
        }
    }
}
```
