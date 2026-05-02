# rust-config-tree

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

`rust-config-tree` 为使用分层配置文件的 Rust 应用提供配置树加载能力和 CLI
辅助能力。

项目手册：<https://developerworks.github.io/rust-config-tree/>。不同语言的手册
作为独立的 mdBook 站点发布，并提供语言切换链接。

它提供：

- 通过 Figment runtime provider 将 `confique` schema 加载成可直接使用的
  config 对象
- `config-template`、`completions` 和 `install-completions` 命令处理
- 生成 Draft 7 root 和 section JSON Schema，供编辑器补全和校验使用
- YAML、TOML、JSON 和 JSON5 配置模板生成
- 为 TOML 和 YAML 模板生成 schema directive，不写入运行时字段
- 递归 include 遍历
- 合并环境变量前加载 `.env`
- 通过 Figment metadata 追踪配置来源
- 通过 `tracing` 输出 TRACE 级别来源追踪日志
- 相对 include 路径从声明它的文件解析
- 词法路径归一化
- include 循环检测
- 确定性遍历顺序
- 镜像模板目标收集
- 按嵌套 schema section 自动拆分 YAML 模板

应用通过派生 `confique::Config` 并实现 `ConfigSchema` 来提供自己的 schema。
`ConfigSchema` 用于暴露 schema 中的 include 字段。

## 安装

```toml
[dependencies]
rust-config-tree = "0.1"
confique = { version = "0.4", features = ["yaml", "toml", "json5"] }
figment = { version = "0.10", features = ["yaml", "env"] }
schemars = { version = "1", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
clap = { version = "4", features = ["derive"] }
```

## 配置结构

应用自己的 schema 持有 include 字段。`rust-config-tree` 只需要一个很小的
adapter，用来从中间 `confique` layer 提取 include。

```rust
use std::path::PathBuf;

use confique::Config;
use rust_config_tree::ConfigSchema;

#[derive(Debug, Config)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(default = "paper")]
    mode: String,

    #[config(nested)]
    server: ServerConfig,
}

#[derive(Debug, Config)]
struct ServerConfig {
    #[config(default = 8080)]
    port: u16,
}

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}
```

相对 include 路径从声明它的文件解析：

```yaml
# config.yaml
include:
  - config/server.yaml

mode: shadow
```

```yaml
# config/server.yaml
server:
  port: 7777
```

使用 `load_config` 加载最终 schema：

```rust
use rust_config_tree::load_config;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = load_config::<AppConfig>("config.yaml")?;
    println!("{config:#?}");

    Ok(())
}
```

`load_config` 会从 root config 所在目录开始向上查找第一个 `.env` 文件并
加载，然后让 Figment 读取 schema 中声明的环境变量。进程里已经存在的环境
变量会保留，并优先于 `.env` 中的值。

运行时配置加载由 Figment 完成。`confique` 仍负责 schema metadata、默认值、
校验和模板生成。环境变量名从 `#[config(env = "...")]` 读取；loader 不使用
`Env::split("_")` 或 `Env::split("__")`，因此 `APP_DATABASE_POOL_SIZE` 可以
映射到 `database.pool_size`，不会把单个 `_` 当成层级分隔符。

`load_config` 不会读取命令行参数，因为 CLI flag 是应用自己的语义。需要 CLI
覆盖配置时，在 `build_config_figment` 之后合并 provider，再通过
`load_config_from_figment` 校验：

CLI flag 名称不会从配置路径自动生成。通常使用应用自己的参数名，比如
`--server-port` 或 `--database-url`；不要依赖 `--server.port` 或 `a.b.c`，
除非应用自己实现了这种 parser。真正决定覆盖哪个配置 key 的，是序列化到
Figment 的嵌套 override 结构。

只有被应用放进 `CliOverrides` provider 的值才会覆盖配置。这个机制面向
频繁临时调整运行参数、但不想修改配置文件的场景。稳定配置仍应放在配置
文件里，只把明确需要临时覆盖的参数暴露成 CLI flag。

```rust
use figment::providers::Serialized;
use serde::Serialize;
use rust_config_tree::{build_config_figment, load_config_from_figment};

#[derive(Debug, Serialize)]
struct CliOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<String>,
}

fn load_with_cli_overrides(cli_mode: Option<String>) -> Result<AppConfig, Box<dyn std::error::Error + Send + Sync>> {
    let cli_overrides = CliOverrides {
        mode: cli_mode,
    };

    let figment = build_config_figment::<AppConfig>("config.yaml")?
        .merge(Serialized::defaults(cli_overrides));

    let config = load_config_from_figment::<AppConfig>(&figment)?;
    Ok(config)
}
```

这样合并 CLI override 后，运行时优先级为：

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```

## 模板生成

模板使用同一份 schema 和 include 遍历规则生成。输出格式由输出路径推断：

- `.yaml` 和 `.yml` 生成 YAML
- `.toml` 生成 TOML
- `.json` 和 `.json5` 生成 JSON5-compatible 模板
- 未知或缺失扩展名生成 YAML

使用 `write_config_schemas` 为 root config 和嵌套 section 生成 Draft 7
JSON Schema。生成的 schema 会移除 `required` 约束，这样 IDE 可以为局部
配置文件提供补全，同时不会因为缺少字段而报错：

```rust
use rust_config_tree::write_config_schemas;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;

    Ok(())
}
```

如果 schema 包含 `server` 和 `log` section，会写入
`schemas/myapp.schema.json`、`schemas/server.schema.json` 和
`schemas/log.schema.json`。root schema 只包含 root 配置文件应该写的字段，
例如 `include` 和 root scalar 字段。它会刻意省略嵌套 section 属性，所以
`server` 和 `log` 只会在编辑各自的 section YAML 文件时补全。

使用 `write_config_templates` 创建 root 模板和 include tree 中的子模板：

```rust
use rust_config_tree::write_config_templates;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;

    Ok(())
}
```

如果生成的 TOML 和 YAML 模板需要绑定这些 schema，用
`write_config_templates_with_schema`：

```rust
use rust_config_tree::write_config_templates_with_schema;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_templates_with_schema::<AppConfig>(
        "config.toml",
        "config.example.toml",
        "schemas/myapp.schema.json",
    )?;

    Ok(())
}
```

root TOML/YAML 目标会绑定 root schema，并且不会补全 child section 字段。
拆分出来的 section YAML 目标会绑定对应的 section schema，例如
`config/log.yaml` 会写入
`# yaml-language-server: $schema=../schemas/log.schema.json`。JSON 和 JSON5
目标不会写 `$schema` 字段；这类文件应通过 VS Code `json.schemas` 等编辑器
设置绑定。

模板生成按这个顺序选择 source tree：

- 已存在的 config path
- 已存在的 output template path
- 将 output path 作为新的空 template tree

## CLI 集成

依赖 `rust-config-tree` 的项目可以保留自己的 clap parser 和命令枚举。
`rust-config-tree` 只提供可复用的 `ConfigCommand` 子命令：

- `config-template`
- `config-schema`
- `config-validate`
- `completions`
- `install-completions`

合并方式如下：

1. 在应用自己的 `Parser` 类型里保留 `#[command(subcommand)] command: Command`。
2. 在应用自己的 `Subcommand` enum 中添加
   `#[command(flatten)] Config(ConfigCommand)`。
3. Clap 会把 flattened variants 展开到应用自己的同一层子命令里。
4. 在 `match` 中处理这个 variant，并调用
   `handle_config_command::<Cli, AppConfig>`。

应用自己的配置覆盖参数仍放在应用自己的 parser 上。例如 `--server-port`
可以通过构造 `CliOverrides { server: Some(CliServerOverrides { port }) }`
映射到 `server.port`，再用 `Serialized::defaults` 合并。

```rust
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use confique::Config;
use schemars::JsonSchema;
use rust_config_tree::{ConfigCommand, ConfigSchema, handle_config_command, load_config};

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,
    #[config(default = "paper")]
    mode: String,
}

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}

#[derive(Debug, Parser)]
#[command(name = "demo")]
struct Cli {
    #[arg(long, default_value = "config.yaml")]
    config: PathBuf,
    #[arg(long)]
    server_port: Option<u16>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Run,
    #[command(flatten)]
    Config(ConfigCommand),
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();
    match cli.command {
        Command::Run => {
            let config = load_config::<AppConfig>(&cli.config)?;
            println!("{config:#?}");
        }
        Command::Config(command) => {
            handle_config_command::<Cli, AppConfig>(command, &cli.config)?;
        }
    }

    Ok(())
}
```

`config-template --output <path>` 将模板写入指定路径。未提供 output path 时，
写入当前目录下的 `config.example.yaml`。添加 `--schema <path>` 后，TOML 和
YAML 模板会绑定生成的 JSON Schema 集合，但不会加入运行时 `$schema` 字段。
这也会把 root schema 和 section schemas 写入指定的 schema path。

`config-schema --output <path>` 写入 root Draft 7 JSON Schema 和 section
schema。未提供 output path 时，root schema 写入
`schemas/config.schema.json`。

`config-validate` 会加载完整 runtime config tree，并执行 `confique` 默认值和
校验。编辑拆分文件时用 editor schema 获得不误报的补全；必填项和最终配置
校验交给这个命令。校验成功时会输出 `Configuration is ok`。

`completions <shell>` 将 completions 输出到 stdout。

`install-completions <shell>` 将 completions 写入用户 home 目录，并在 shell
需要时更新启动文件。支持 Bash、Elvish、Fish、PowerShell 和 Zsh。

## 低层 Tree API

不使用 `confique`，或者需要直接访问遍历结果时，可以使用 `load_config_tree`：

```rust
use std::{fs, io, path::{Path, PathBuf}};

use rust_config_tree::{ConfigSource, load_config_tree};

fn load_source(path: &Path) -> io::Result<ConfigSource<String>> {
    let content = fs::read_to_string(path)?;
    let includes = content
        .lines()
        .filter_map(|line| line.strip_prefix("include: "))
        .map(PathBuf::from)
        .collect();

    Ok(ConfigSource::new(content, includes))
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let tree = load_config_tree("config.yaml", load_source)?;

    for node in tree.nodes() {
        println!("{}", node.path().display());
    }

    Ok(())
}
```

Tree API 会进行词法路径归一化、拒绝空 include path、检测递归 include 循环，
并跳过已经从其他 include 分支加载过的文件。

## 许可证

按你的选择使用以下任一许可证：

- Apache License, Version 2.0
- MIT license
