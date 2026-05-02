# 快速开始

[English](../en/quick-start.html) | [中文](quick-start.html)

添加 crate 以及应用所需的 schema 和运行时库：

```toml
[dependencies]
rust-config-tree = "0.1"
confique = { version = "0.4", features = ["yaml", "toml", "json5"] }
figment = { version = "0.10", features = ["yaml", "toml", "json", "env"] }
schemars = { version = "1", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
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

默认 `load_config` 优先级为：

```text
environment variables
  > config files, with later merged files overriding earlier files
    > confique code defaults
```

通过高层 API 加载 include 时，root 文件拥有最高的文件优先级。被 include
的文件优先级更低，适合承载默认值或按 section 拆分的配置。

命令行参数属于应用自己的 CLI 语义，所以 `load_config` 不会自动读取。应用
有配置覆盖参数时，在 `build_config_figment` 之后合并 CLI override：

CLI flag 名称由应用自己决定，不会自动使用 `a.b.c` 配置路径。推荐使用
正常的 clap 参数名，比如 `--server-port`，再映射成嵌套 override 结构。
真正决定覆盖哪个配置 key 的，是序列化后的嵌套结构。

只有被应用放进 `CliOverrides` provider 的值才会覆盖配置。这个机制适合
单次运行频繁调整参数、但不想修改配置文件的场景。稳定值应继续保存在
配置文件中。

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

这样合并 CLI override 后，完整优先级为：

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```
