# CLI 集成

[English](../en/cli.html) | [中文](cli.html) | [日本語](../ja/cli.html) | [한국어](../ko/cli.html) | [Français](../fr/cli.html) | [Deutsch](../de/cli.html) | [Español](../es/cli.html) | [Português](../pt/cli.html) | [Svenska](../sv/cli.html) | [Suomi](../fi/cli.html) | [Nederlands](../nl/cli.html)

`ConfigCommand` 提供可复用的 clap 子命令：

- `config-template`
- `config-schema`
- `config-validate`
- `completions`
- `install-completions`
- `uninstall-completions`

这些内置子命令不同于应用自己的配置覆盖参数。配置覆盖参数应在运行时加载
路径里作为 Figment provider 合并。

配置覆盖参数仍属于依赖方应用自己的 CLI。参数名不需要匹配点分配置路径。
例如应用可以解析 `--server-port`，再把它映射到嵌套配置 key `server.port`。
只有应用映射进 `CliOverrides` 的 flag 才会影响配置值。

将它 flatten 到应用命令枚举中：

1. 保留应用自己的 `Parser` 类型。
2. 保留应用自己的 `Subcommand` enum。
3. 在这个 enum 里添加 `#[command(flatten)] Config(ConfigCommand)`。
4. Clap 会把 flattened `ConfigCommand` variants 展开到应用自己的同一层命令。
5. 在 `match` 里处理 `Config(command)` variant，并交给
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
路径，只使用其中的文件名。未提供 output file name 时，命令写入
`config/<root_config_name>/<root_config_name>.example.yaml`。添加
`--schema schemas/myapp.schema.json` 后，生成的 TOML 和 YAML 模板会绑定生成的
JSON Schema。拆分出的 YAML 模板会绑定对应的 section schema。该命令也会把
root 和 section schemas 写入指定的 schema path。

```bash
demo config-template --output app_config.example.toml --schema schemas/myapp.schema.json
```

生成 root 和 section JSON Schema：

```bash
demo config-schema --output schemas/myapp.schema.json
```

未提供 `--output` 时，`config-schema` 会把 root schema 写入
`config/<root_config_name>/<root_config_name>.schema.json`。

校验完整 runtime config tree：

```bash
demo config-validate
```

生成的 editor schema 会刻意避免在拆分文件里触发必填字段诊断。
`config-validate` 会加载 includes、应用默认值，并执行最终 `confique` 校验，
包括通过 `#[config(validate = Self::validate)]` 声明的校验。生成的
`*.schema.json` 仍然只用于 IDE 补全和基础编辑期检查，不负责字段值合法性判断。
校验成功时会输出 `Configuration is ok`。

## Shell Completions

输出 completions 到 stdout：

```bash
demo completions zsh
```

安装 completions：

```bash
demo install-completions zsh
```

卸载 completions：

```bash
demo uninstall-completions zsh
```

安装器支持 Bash、Elvish、Fish、PowerShell 和 Zsh。它会将 completion 文件
写入用户 home 目录，并为需要显式配置的 shell 更新启动文件。

在修改已有 shell 启动文件之前，例如 `~/.zshrc`、`~/.bashrc`、Elvish rc
文件或 PowerShell profile，命令会先在原文件旁边写入备份：

```text
<rc-file>.backup.by.<program-name>.<timestamp>
```
