# 快速开始

[English](../en/quick-start.html) | [中文](quick-start.html)

添加 crate 以及应用所需的 schema 和运行时库：

```toml
[dependencies]
rust-config-tree = "0.1"
confique = { version = "0.4", features = ["yaml", "toml", "json5"] }
figment = { version = "0.10", features = ["yaml", "toml", "json", "env"] }
clap = { version = "4", features = ["derive"] }
```

定义 `confique` schema，并为 root 类型实现 `ConfigSchema`：

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

加载配置：

```rust
use rust_config_tree::load_config;

let config = load_config::<AppConfig>("config.yaml")?;
println!("{config:#?}");
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

root 文件可以递归 include 子配置：

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

运行时优先级为：

```text
environment variables
  > config files, with later merged files overriding earlier files
    > confique code defaults
```

通过高层 API 加载 include 时，root 文件拥有最高的文件优先级。被 include
的文件优先级更低，适合承载默认值或按 section 拆分的配置。
