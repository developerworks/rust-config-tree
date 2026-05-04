# CLI(命令行接口) 集成

[English](../en/cli.html) | [中文](cli.html) | [日本語](../ja/cli.html) | [한국어](../ko/cli.html) | [Français](../fr/cli.html) | [Deutsch](../de/cli.html) | [Español](../es/cli.html) | [Português](../pt/cli.html) | [Svenska](../sv/cli.html) | [Suomi](../fi/cli.html) | [Nederlands](../nl/cli.html)

`ConfigCommand` 提供以下可复用的 clap(命令行解析库) 子命令：

- `config-template` 会生成配置模板。
- `config-schema` 会生成 JSON Schema(JSON 结构定义)。
- `config-validate` 会校验最终配置。
- `completions` 会输出 completion(补全脚本)。
- `install-completions` 会安装 completion(补全脚本)。
- `uninstall-completions` 会卸载 completion(补全脚本)。

这些内置子命令不同于应用自己的配置覆盖参数。配置覆盖参数应在运行时加载
路径里作为 Figment(配置合并库) provider(值提供器) 合并。

配置覆盖参数仍属于依赖方应用自己的 CLI(命令行接口)。参数名不需要匹配点分配置路径。
例如，应用可以解析 `--server-port`，再把它映射到嵌套配置 key(键) `server.port`。
只有应用映射进 `CliOverrides` 的 flag(命令行参数) 才会影响配置值。

应用可以把 `ConfigCommand` flatten(展开) 到自己的命令枚举中：

1. 保留应用自己的 `Parser` 类型。
2. 保留应用自己的 `Subcommand` enum。
3. 在这个 enum 里添加 `#[command(flatten)] Config(ConfigCommand)`。
4. Clap(命令行解析库) 会把 flattened(已展开的) `ConfigCommand` variants(变体)
   展开到应用自己的同一层命令。
5. 应用在 `match` 里处理 `Config(command)` variant(变体)，并交给
   `handle_config_command`。

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

## 配置模板

```bash
demo config-template --output app_config.example.yaml
```

命令会在 `config/<root_config_name>/` 下写入模板。如果 `--output` 接收到
路径，命令只使用其中的文件名。未提供 output file name(输出文件名) 时，命令写入
`config/<root_config_name>/<root_config_name>.example.yaml`。添加
`--schema schemas/myapp.schema.json` 后，生成的 TOML 和 YAML 模板会绑定生成的
JSON Schema(JSON 结构定义)。拆分出的 YAML 模板会绑定对应的
section schema(配置段结构定义)。该命令也会把 root(根配置) 和
section schema(配置段结构定义) 写入指定的 schema path(结构定义路径)。

```bash
demo config-template --output app_config.example.toml --schema schemas/myapp.schema.json
```

下面的命令会生成 root(根配置) 和 section(配置段) 的 JSON Schema(JSON 结构定义)：

```bash
demo config-schema --output schemas/myapp.schema.json
```

未提供 `--output` 时，`config-schema` 会把 root schema(根结构定义) 写入
`config/<root_config_name>/<root_config_name>.schema.json`。

下面的命令会校验完整的 runtime config tree(运行时配置树)：

```bash
demo config-validate
```

生成的 editor schema(编辑器结构定义) 会刻意避免在拆分文件里触发必填字段诊断。
`config-validate` 会加载 includes(包含文件)、应用默认值，并执行最终
`confique` 校验，
包括通过 `#[config(validate = Self::validate)]` 声明的校验。生成的
`*.schema.json` 仍然只用于 IDE(集成开发环境) 补全和基础编辑期检查，不负责字段值合法性判断。
校验成功时会输出 `Configuration is ok`。

## Shell Completions(命令行补全)

下面的命令会把 completions(补全脚本) 输出到 stdout(标准输出)：

```bash
demo completions zsh
```

下面的命令会安装 completions(补全脚本)：

```bash
demo install-completions zsh
```

下面的命令会卸载 completions(补全脚本)：

```bash
demo uninstall-completions zsh
```

安装器支持 Bash、Elvish、Fish、PowerShell 和 Zsh。它会将 completion(补全脚本)
文件写入用户 home(主目录) 目录，并为需要显式配置的 shell(命令行外壳)
更新启动文件。

在修改已有 shell(命令行外壳) 启动文件之前，例如 `~/.zshrc`、`~/.bashrc`、
Elvish rc(运行控制) 文件或 PowerShell profile(配置文件)，命令会先在原文件旁边写入备份：

```text
<rc-file>.backup.by.<program-name>.<timestamp>
```
