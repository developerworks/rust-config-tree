# CLI 統合

[English](../en/cli.html) | [中文](../zh/cli.html) | [日本語](cli.html) | [한국어](../ko/cli.html) | [Français](../fr/cli.html) | [Deutsch](../de/cli.html) | [Español](../es/cli.html) | [Português](../pt/cli.html) | [Svenska](../sv/cli.html) | [Suomi](../fi/cli.html) | [Nederlands](../nl/cli.html)

`ConfigCommand` は reusable clap subcommands を提供します。

- `config-template`
- `config-schema`
- `config-validate`
- `completions`
- `install-completions`

これらの built-in subcommands は application-specific config override flags とは
別のものです。config override flags は runtime loading path で Figment provider
として merge します。

config override flags は consuming application の CLI に属します。名前は dotted
config path に一致している必要はありません。たとえば application は
`--server-port` を parse し、それを nested `server.port` config key に map
できます。`CliOverrides` に map した flag だけが config value に影響します。

application command enum に flatten します。

1. application 自身の `Parser` type を保つ。
2. application 自身の `Subcommand` enum を保つ。
3. その enum に `#[command(flatten)] Config(ConfigCommand)` を追加する。
4. Clap は flattened `ConfigCommand` variants を application 自身の command と
   同じ level に展開する。
5. `Config(command)` variant を match し、`handle_config_command` に渡す。

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

## Config Templates

```bash
demo config-template --output config.example.yaml
```

output path がない場合、この command は current directory に
`config.example.yaml` を書きます。`--schema schemas/myapp.schema.json` を追加
すると、generated TOML / YAML template を generated JSON Schema に bind します。
split YAML template は matching section schema を bind します。この command は
root / section schema も selected schema path に書きます。

```bash
demo config-template --output config.example.toml --schema schemas/myapp.schema.json
```

root / section JSON Schema を生成します。

```bash
demo config-schema --output schemas/myapp.schema.json
```

完全な runtime config tree を validate します。

```bash
demo config-validate
```

generated editor schemas は split file で required-field diagnostic を避けるよう
に作られます。`config-validate` は includes を読み込み、defaults を適用し、
final `confique` validation を実行します。成功時は `Configuration is ok` を
出力します。

## Shell Completions

completion を stdout に出力します。

```bash
demo completions zsh
```

completion を install します。

```bash
demo install-completions zsh
```

installer は Bash、Elvish、Fish、PowerShell、Zsh を support します。completion
file を user home directory 以下に書き、必要な shell では startup file も更新
します。

