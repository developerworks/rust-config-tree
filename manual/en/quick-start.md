# Quick Start

[English](quick-start.html) | [中文](../zh/quick-start.html)

Add the crate and the schema/runtime libraries used by your application:

```toml
[dependencies]
rust-config-tree = "0.1"
confique = { version = "0.4", features = ["yaml", "toml", "json5"] }
figment = { version = "0.10", features = ["yaml", "toml", "json", "env"] }
schemars = { version = "1", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
clap = { version = "4", features = ["derive"] }
```

Define a `confique` schema and implement `ConfigSchema` for the root type:

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

Load the config:

```rust
use rust_config_tree::load_config;

let config = load_config::<AppConfig>("config.yaml")?;
println!("{config:#?}");
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Use a root file with recursive includes:

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

The default `load_config` precedence is:

```text
environment variables
  > config files, with later merged files overriding earlier files
    > confique code defaults
```

When includes are loaded by the high-level API, the root file has the highest
file priority. Included files provide lower-priority values and can be used for
defaults or section-specific files.

Command-line arguments are application-specific, so `load_config` does not read
them automatically. Merge CLI overrides after `build_config_figment` when the
application has config override flags:

CLI flag names are chosen by the application. They are not automatically
`a.b.c` config paths. Prefer normal clap flags such as `--server-port`, then
map them into a nested override structure. The nested serialized shape controls
the config key that is overridden.

Only values represented in the application's `CliOverrides` provider override
configuration. This is useful for parameters that are changed frequently for one
run without editing the config file. Stable values should stay in config files.

```rust
use figment::providers::Serialized;
use serde::Serialize;
use rust_config_tree::{build_config_figment, load_config_from_figment};

#[derive(Debug, Serialize)]
struct CliOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    server: Option<CliServerOverrides>,
}

#[derive(Debug, Serialize)]
struct CliServerOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    port: Option<u16>,
}

let cli_overrides = CliOverrides {
    server: Some(CliServerOverrides { port: Some(9000) }),
};

let figment = build_config_figment::<AppConfig>("config.yaml")?
    .merge(Serialized::defaults(cli_overrides));

let config = load_config_from_figment::<AppConfig>(&figment)?;
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

With CLI overrides merged this way, the full precedence is:

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```
