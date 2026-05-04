# 設定スキーマ

[English](../en/schema.html) | [中文](../zh/schema.html) | [日本語](schema.html) | [한국어](../ko/schema.html) | [Français](../fr/schema.html) | [Deutsch](../de/schema.html) | [Español](../es/schema.html) | [Português](../pt/schema.html) | [Svenska](../sv/schema.html) | [Suomi](../fi/schema.html) | [Nederlands](../nl/schema.html)

application schema は通常の `confique` config type です。root schema は
`ConfigSchema` を実装する必要があります。これにより `rust-config-tree` は
中間 `confique` layer から recursive include を見つけられます。

```rust
use std::path::PathBuf;

use confique::Config;
use schemars::JsonSchema;
use rust_config_tree::ConfigSchema;

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    database: DatabaseConfig,
}

#[derive(Debug, Config, JsonSchema)]
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
loading では常に使われます。独立した `config/*.yaml` template と
`schemas/*.schema.json` schema も生成したい nested field には
`#[schemars(extend("x-tree-split" = true))]` を追加します。

```rust
#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    server: ServerConfig,
}
```

自然な YAML shape は次の通りです。

```yaml
server:
  bind: 127.0.0.1
  port: 8080
```

## 環境変数専用フィールド

値を環境変数からだけ渡し、生成される config file には出したくない leaf field には `#[schemars(extend("x-env-only" = true))]` を付けます。生成される YAML template と JSON Schema は env-only field を省略し、その結果空になった parent object も削除します.

```rust
#[config(env = "APP_SECRET")]
#[schemars(extend("x-env-only" = true))]
secret: String,
```

## Field Value Validation

生成された `*.schema.json` file は IDE 補完と基本的な editor check のための
ものです。具体的な field value が application として合法かどうかは判断しません。

field value validation は code 側で `#[config(validate = Self::validate)]` として
実装します。final config を `load_config` で読み込むとき、または
`config-validate` で確認するときに、この runtime validation が実行されます。

## Template Section Overrides

template source に include がない場合、crate は `x-tree-split` で mark した nested schema section から child
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
