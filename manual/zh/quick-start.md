# 快速开始

[English](../en/quick-start.html) | [中文](quick-start.html) | [日本語](../ja/quick-start.html) | [한국어](../ko/quick-start.html) | [Français](../fr/quick-start.html) | [Deutsch](../de/quick-start.html) | [Español](../es/quick-start.html) | [Português](../pt/quick-start.html) | [Svenska](../sv/quick-start.html) | [Suomi](../fi/quick-start.html) | [Nederlands](../nl/quick-start.html)

先添加 crate(软件包)，再添加应用所需的 schema(结构定义) 库和运行时库：

```toml
[dependencies]
rust-config-tree = "0.1"
confique = { version = "0.4", features = ["yaml", "toml", "json5"] }
figment = { version = "0.10", features = ["yaml", "toml", "json", "env"] }
schemars = { version = "1", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
clap = { version = "4", features = ["derive"] }
```

然后定义 `confique` schema(结构定义)，并为 root(根配置) 类型实现
`ConfigSchema`：

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

root(根配置) 文件可以通过 include(包含) 递归加载子配置：

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

默认情况下，`load_config` 使用以下优先级：

```text
环境变量
  > 配置文件，后合并的文件会覆盖先合并的文件
    > confique 代码默认值
```

通过高层 API(应用程序接口) 加载 include(包含文件) 时，root(根配置) 文件拥有
最高的文件优先级。被 include(包含) 的文件优先级更低，适合承载默认值，也适合
承载按 section(配置段) 拆分的配置。

命令行参数属于应用自己的 CLI(命令行接口) 语义，所以 `load_config` 不会自动读取。
当应用需要用命令行参数覆盖配置时，应用应在 `build_config_figment` 之后合并
CLI override(命令行覆盖值)。合并方式如下：

CLI flag(命令行参数) 名称由应用自己决定，加载器不会自动使用 `a.b.c` 这种
配置路径。推荐使用正常的 clap(命令行解析库) 参数名，比如 `--server-port`，
再把参数值映射成嵌套 override(覆盖值) 结构。序列化后的嵌套结构真正决定
哪个配置 key(键) 会被覆盖。

只有应用放进 `CliOverrides` provider(值提供器) 的值才会覆盖配置。这个机制
适合单次运行时频繁调整参数、但不想修改配置文件的场景。稳定值应继续保存在
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

这样合并 CLI override(命令行覆盖值) 后，完整优先级如下：

```text
命令行覆盖值
  > 环境变量
    > 配置文件
      > confique 代码默认值
```
