# rust-config-tree

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

`rust-config-tree` 为使用分层配置文件的 Rust(系统编程语言) 应用提供配置树加载
能力和 CLI(命令行接口) 辅助能力。

项目手册：<https://developerworks.github.io/rust-config-tree/>。不同语言的手册
作为独立的 mdBook(文档构建工具) 站点发布，并提供语言切换链接。

它提供以下能力：

- 它会通过 Figment(配置合并库) runtime provider(运行时值提供器) 将 `confique`
  schema(结构定义) 加载成可直接使用的 config(配置) 对象。
- 它会处理 `config-template`、`config-schema`、`config-validate`、`completions`、
  `install-completions` 和 `uninstall-completions` 命令。
- 它会生成 Draft 7 root(根配置) 和 section(配置段) 的
  JSON Schema(JSON 结构定义)，供编辑器补全和基础 schema(结构定义) 检查使用。
- 它会生成 YAML、TOML、JSON 和 JSON5 配置模板。
- 它会为 TOML、YAML、JSON 和 JSON5 模板生成 schema(结构定义) 绑定。
- 它会递归遍历 include(包含文件)。
- 它会在合并环境变量之前加载 `.env` 文件。
- 它会通过 Figment(配置合并库) metadata(元数据) 追踪配置来源。
- 它会通过 `tracing` 输出 TRACE(追踪级别) 来源追踪日志。
- 它会从声明 include(包含) 的文件解析相对路径。
- 它会执行词法路径归一化。
- 它会检测 include(包含) 循环。
- 它会使用确定性的遍历顺序。
- 它会收集镜像模板目标。
- 它会按显式标记的嵌套 schema section(结构定义配置段) 拆分 YAML 模板。

应用通过派生 `confique::Config` 并实现 `ConfigSchema` 来提供自己的
schema(结构定义)。`ConfigSchema` 用于暴露 schema(结构定义) 中的
include(包含) 字段。

## 安装

```toml
[dependencies]
rust-config-tree = "0.1"
confique = { version = "0.4", features = ["yaml", "toml", "json5"] }
figment = { version = "0.10", features = ["yaml", "toml", "json", "env"] }
schemars = { version = "1", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
clap = { version = "4", features = ["derive"] }
```

## 配置结构

应用自己的 schema(结构定义) 持有 include(包含) 字段。`rust-config-tree` 只需要
一个很小的 adapter(适配器)，用来从中间 `confique` layer(层) 提取
include(包含)。

```rust
use std::path::PathBuf;

use confique::Config;
use schemars::JsonSchema;
use rust_config_tree::ConfigSchema;

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(default = "paper")]
    mode: String,

    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    server: ServerConfig,
}

#[derive(Debug, Config, JsonSchema)]
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

加载器会从声明 include(包含) 的文件解析相对 include(包含) 路径：

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

使用 `load_config` 可以加载最终 schema(结构定义)：

```rust
use rust_config_tree::load_config;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = load_config::<AppConfig>("config.yaml")?;
    println!("{config:#?}");

    Ok(())
}
```

`load_config` 会从 root config(根配置) 所在目录开始向上查找第一个 `.env`
文件并加载，然后让 Figment(配置合并库) 读取 schema(结构定义) 中声明的环境变量。
进程里已经存在的环境变量会保留，并优先于 `.env` 中的值。

运行时配置加载由 Figment(配置合并库) 完成。`confique` 仍负责
schema metadata(结构定义元数据)、默认值、校验和模板生成。环境变量名从
`#[config(env = "...")]` 读取；loader(加载器) 不使用
`Env::split("_")` 或 `Env::split("__")`，因此 `APP_DATABASE_POOL_SIZE` 可以
映射到 `database.pool_size`，不会把单个 `_` 当成层级分隔符。

`load_config` 不会读取命令行参数，因为 CLI flag(命令行参数) 是应用自己的语义。
需要用 CLI(命令行接口) 覆盖配置时，应用应在 `build_config_figment` 之后合并
provider(值提供器)，再通过
`load_config_from_figment` 校验：

CLI flag(命令行参数) 名称不会从配置路径自动生成。通常使用应用自己的参数名，比如
`--server-port` 或 `--database-url`；不要依赖 `--server.port` 或 `a.b.c`，
除非应用自己实现了这种 parser(解析器)。真正决定覆盖哪个配置 key(键) 的，
是序列化到 Figment(配置合并库) 的嵌套 override(覆盖值) 结构。

只有被应用放进 `CliOverrides` provider(值提供器) 的值才会覆盖配置。这个机制面向
频繁临时调整运行参数、但不想修改配置文件的场景。稳定配置仍应放在配置
文件里，只把明确需要临时覆盖的参数暴露成 CLI flag(命令行参数)。

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

这样合并 CLI override(命令行覆盖值) 后，运行时优先级如下：

```text
命令行覆盖值
  > 环境变量
    > 配置文件
      > confique 代码默认值
```

## 模板生成

模板使用同一份 schema(结构定义) 和 include(包含文件) 遍历规则生成。输出格式由
输出路径推断：

- `.yaml` 和 `.yml` 会生成 YAML。
- `.toml` 会生成 TOML。
- `.json` 和 `.json5` 会生成 JSON5-compatible(JSON5 兼容) 模板。
- 未知或缺失扩展名会生成 YAML。

使用 `write_config_schemas` 可以为 root config(根配置) 和显式拆分的嵌套
section(配置段) 生成 Draft 7 JSON Schema(JSON 结构定义)。如果 nested(嵌套)
字段需要独立生成 `*.yaml` 和 `<section>.schema.json`，就使用
`#[schemars(extend("x-tree-split" = true))]` 标记这个字段。没有这个标记的
nested(嵌套) 字段会留在父模板和父 schema(结构定义) 中。

当某个 leaf(叶子) 字段只能从环境变量提供时，可以添加
`#[schemars(extend("x-env-only" = true))]`。生成的模板和
JSON Schema(JSON 结构定义) 会省略 env-only(仅环境变量) 字段；如果父对象因此变空，
生成器也会删除这个父对象。

生成的 schema(结构定义) 会移除 `required` 约束，这样 IDE(集成开发环境)
可以为局部配置文件提供补全，同时不会因为缺少字段而报错。生成的
`*.schema.json` 文件只用于 IDE(集成开发环境) 补全和基础编辑期检查，不负责判断
具体字段值对应用是否合法。字段值合法性应在代码中通过
`#[config(validate = Self::validate)]` 实现，并由 `load_config` 或
`config-validate` 触发：

```rust
use rust_config_tree::write_config_schemas;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;

    Ok(())
}
```

如果 schema(结构定义) 中的 `server` 和 `log` section(配置段) 标记了
`x-tree-split`，会写入
`schemas/myapp.schema.json`、`schemas/server.schema.json` 和
`schemas/log.schema.json`。root schema(根结构定义) 只包含 root(根配置)
配置文件应该写的字段，例如 `include` 和 root scalar(根标量) 字段。它会刻意省略
被拆分的嵌套 section(配置段) 属性，所以 `server` 和 `log` 只会在编辑各自的
section(配置段) YAML 文件时补全。没有 `x-tree-split` 的 nested section(嵌套配置段)
会保留在 root schema(根结构定义) 中，因为它们没有独立的模板文件和
schema(结构定义) 文件。

使用 `write_config_templates` 可以创建 root(根配置) 模板和 include tree(包含树)
中的子模板：

```rust
use rust_config_tree::write_config_templates;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;

    Ok(())
}
```

如果生成的 TOML、YAML、JSON 和 JSON5 模板需要绑定这些 schema(结构定义)，可以使用
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

root(根配置) 目标会绑定 root schema(根结构定义)，并且不会补全被拆分的
child section(子配置段) 字段。拆分出来的 section(配置段) YAML 目标会绑定对应的
section schema(配置段结构定义)，例如
`log.yaml` 会写入
`# yaml-language-server: $schema=./schemas/log.schema.json`。JSON 和 JSON5
目标会写入顶层 `$schema` 字段，指向匹配的生成 schema(结构定义)。
VS Code(代码编辑器) `json.schemas` 等编辑器设置仍可作为替代绑定方式。

模板生成会按以下顺序选择 source tree(来源树)：

- 它会先使用已存在的 config path(配置路径)。
- 它会再使用已存在的 output template path(输出模板路径)。
- 它最后会把 output path(输出路径) 当作新的空 template tree(模板树)。

## CLI(命令行接口) 集成

依赖 `rust-config-tree` 的项目可以保留自己的 clap parser(命令行解析器) 和命令枚举。
`rust-config-tree` 只提供以下可复用的 `ConfigCommand` 子命令：

- `config-template` 会生成配置模板。
- `config-schema` 会生成 JSON Schema(JSON 结构定义)。
- `config-validate` 会校验最终配置。
- `completions` 会输出 completion(补全脚本)。
- `install-completions` 会安装 completion(补全脚本)。
- `uninstall-completions` 会卸载 completion(补全脚本)。

合并方式如下：

1. 在应用自己的 `Parser` 类型里保留 `#[command(subcommand)] command: Command`。
2. 应用在自己的 `Subcommand` enum 中添加
   `#[command(flatten)] Config(ConfigCommand)`。
3. Clap(命令行解析库) 会把 flattened variants(已展开变体) 展开到应用自己的同一层子命令里。
4. 应用在 `match` 中处理这个 variant(变体)，并调用
   `handle_config_command::<Cli, AppConfig>`。

应用自己的配置覆盖参数仍放在应用自己的 parser(解析器) 上。例如 `--server-port`
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

`config-template --output <file-name>` 会在 `config/<root_config_name>/`
下写入模板，并使用指定的文件名。如果传入的是路径，只取它的文件名。未提供
output file name(输出文件名) 时，写入
`config/<root_config_name>/<root_config_name>.example.yaml`。添加
`--schema <path>` 后，TOML、YAML、JSON 和 JSON5 模板会绑定生成的
JSON Schema 集合。这也会把 root schema(根结构定义) 和
section schema(配置段结构定义) 写入指定的 schema path(结构定义路径)。

`config-schema --output <path>` 会写入 root(根配置) 的 Draft 7
JSON Schema(JSON 结构定义) 和 section schema(配置段结构定义)。未提供
output path(输出路径) 时，root schema(根结构定义) 写入
`config/<root_config_name>/<root_config_name>.schema.json`。

`config-validate` 会加载完整 runtime config tree(运行时配置树)，并执行
`confique` 默认值和
校验，包括通过 `#[config(validate = Self::validate)]` 声明的校验。编辑拆分
文件时，可以用 editor schema(编辑器结构定义) 获得不误报的补全；必填项和最终
配置校验应交给这个命令。校验成功时会输出 `Configuration is ok`。

`completions <shell>` 会将 completions(补全脚本) 输出到 stdout(标准输出)。

`install-completions <shell>` 会将 completions(补全脚本) 写入用户 home(主目录)，
并在 shell(命令行外壳) 需要时更新启动文件。它支持 Bash、Elvish、Fish、
PowerShell 和 Zsh。

`uninstall-completions <shell>` 会删除当前 binary(二进制程序) 的
completion(补全脚本) 文件，并在 shell(命令行外壳) 使用
managed startup block(托管启动块) 时删除这个 block(块)。

## 低层 Tree API(树形接口)

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

Tree API(树形接口) 会进行词法路径归一化、拒绝空 include path(包含路径)、
检测递归 include(包含) 循环，并跳过已经从其他 include(包含) 分支加载过的文件。

## 许可证

按你的选择使用以下任一许可证：

- Apache License, Version 2.0(Apache 2.0 许可证)
- MIT license(MIT 许可证)
