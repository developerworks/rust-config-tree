# 設定スキーマ

[English](../en/schema.html) | [中文](../zh/schema.html) | [日本語](schema.html) | [한국어](../ko/schema.html) | [Français](../fr/schema.html) | [Deutsch](../de/schema.html) | [Español](../es/schema.html) | [Português](../pt/schema.html) | [Svenska](../sv/schema.html) | [Suomi](../fi/schema.html) | [Nederlands](../nl/schema.html)

application schema は通常の `confique` config type です。root schema は
`ConfigSchema` を実装する必要があります。これにより `rust-config-tree` は
中間 `confique` layer から recursive include を見つけられます。

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

## Include Field

include field の名前は任意です。`rust-config-tree` は
`ConfigSchema::include_paths` を通してのみ include field を知ります。

通常、この field には empty default を付けます。

```rust
#[config(default = [])]
include: Vec<PathBuf>,
```

loader は各 file を partially loaded layer として受け取ります。これにより、
最終 schema の merge / validation より前に child config files を発見できます。

## Nested Sections

structured section には `#[config(nested)]` を使います。nested section は runtime
loading と template splitting の両方で重要です。

```rust
#[derive(Debug, Config)]
struct AppConfig {
    #[config(nested)]
    server: ServerConfig,
}
```

自然な YAML shape は次の通りです。

```yaml
server:
  bind: 127.0.0.1
  port: 8080
```

## Template Section Overrides

template source に include がない場合、crate は nested schema section から child
template file を導出できます。default top-level path は
`config/<section>.yaml` です。

`template_path_for_section` で path を上書きできます。

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

