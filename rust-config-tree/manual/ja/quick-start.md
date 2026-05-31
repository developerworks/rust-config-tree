# クイックスタート

[English](../en/quick-start.html) | [中文](../zh/quick-start.html) | [日本語](quick-start.html) | [한국어](../ko/quick-start.html) | [Français](../fr/quick-start.html) | [Deutsch](../de/quick-start.html) | [Español](../es/quick-start.html) | [Português](../pt/quick-start.html) | [Svenska](../sv/quick-start.html) | [Suomi](../fi/quick-start.html) | [Nederlands](../nl/quick-start.html)

crate と、application が使う schema/runtime library を追加します。

```toml
[dependencies]
rust-config-tree = "0.2"
confique = { version = "0.4", features = ["yaml", "toml", "json5"] }
figment = { version = "0.10", features = ["yaml", "toml", "json", "env"] }
schemars = { version = "1", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
clap = { version = "4", features = ["derive"] }
```

`confique` schema を定義し、root type に `ConfigSchema` を実装します。

```rust
use std::path::PathBuf;

use confique::Config;
use rust_config_tree::ConfigSchema;

#[derive(Debug, Config)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(nested)]
    server: ServerConfig,
}

#[derive(Debug, Config)]
struct ServerConfig {
    #[config(default = "127.0.0.1")]
    #[config(env = "APP_SERVER_BIND")]
    bind: String,

    #[config(default = 8080)]
    #[config(env = "APP_SERVER_PORT")]
    port: u16,
}

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}
```

config を読み込みます。

```rust
use rust_config_tree::load_config;

let config = load_config::<AppConfig>("config.yaml")?;
println!("{config:#?}");
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

root file は recursive include を持てます。

```yaml
# config.yaml
include:
  - config/server.yaml
```

```yaml
# config/server.yaml
server:
  bind: 0.0.0.0
  port: 3000
```

default の `load_config` precedence は次の通りです。

```text
environment variables
  > config files, with later merged files overriding earlier files
    > confique code defaults
```

high-level API で includes を読み込む場合、root file が最も高い file priority
を持ちます。included files は lower-priority value を提供し、default や
section-specific file として使えます。

command-line arguments は application-specific なので、`load_config` は自動で
読みません。application が config override flag を持つ場合は、
`build_config_figment` の後に CLI override を merge します。

command-line arguments は application-specific なので、`load_config` は自動で
読みません。`ConfigOverrides` derive マクロを使って、パース済みの CLI フラグ
からオーバーライドプロバイダを構築します：

```rust
use clap::Parser;
use rust_config_tree::{
    ConfigSchema,
    cli::ConfigOverrides,
    config::{build_config_figment, load_config_from_figment},
};

#[derive(Debug, Parser, ConfigOverrides)]
struct Cli {
    #[arg(long)]
    config: Option<std::path::PathBuf>,

    #[arg(long)]
    #[config_override(path = "server.port")]
    server_port: Option<u16>,

    #[arg(long)]
    #[config_override(path = "log.level")]
    log_level: Option<String>,
}

let cli = Cli::parse();
let figment = build_config_figment::<AppConfig>("config.yaml")?
    .merge(cli.config_overrides()?);
let config = load_config_from_figment::<AppConfig>(&figment)?;
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

`#[config_override(path = "...")]` 属性は各 CLI フラグをドット区切りの設定
パスにマッピングします。指定されたフラグのみがオーバーライド値を生成し、
省略されたフラグは無視されます。オーバーライドプロバイダは最後にマージされる
ため、指定されたフラグはファイルや環境変数の値を上書きします：

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```
