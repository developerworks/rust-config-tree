# CLI 集成

[English](../en/cli.html) | [中文](cli.html)

`ConfigCommand` 提供可复用的 clap 子命令：

- `config-template`
- `completions`
- `install-completions`

将它 flatten 到应用命令枚举中：

```rust
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use confique::Config;
use rust_config_tree::{ConfigCommand, ConfigSchema, handle_config_command, load_config};

#[derive(Debug, Config)]
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
demo config-template --output config.example.yaml
```

如果未提供 output path，命令会在当前目录写入 `config.example.yaml`。

## Shell Completions

输出 completions 到 stdout：

```bash
demo completions zsh
```

安装 completions：

```bash
demo install-completions zsh
```

安装器支持 Bash、Elvish、Fish、PowerShell 和 Zsh。它会将 completion 文件
写入用户 home 目录，并为需要显式配置的 shell 更新启动文件。
