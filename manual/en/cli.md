# CLI Integration

[English](cli.html) | [中文](../zh/cli.html) | [日本語](../ja/cli.html) | [한국어](../ko/cli.html) | [Français](../fr/cli.html) | [Deutsch](../de/cli.html) | [Español](../es/cli.html) | [Português](../pt/cli.html) | [Svenska](../sv/cli.html) | [Suomi](../fi/cli.html) | [Nederlands](../nl/cli.html)

`ConfigCommand` provides reusable clap subcommands:

- `config-template`
- `config-schema`
- `config-validate`
- `completions`
- `install-completions`

These built-in subcommands are separate from application-specific config
override flags. Merge config override flags as Figment providers in the runtime
loading path.

Config override flags remain part of the consuming application's CLI. Their
names do not need to match dotted config paths. For example, the application can
parse `--server-port` and map it to the nested `server.port` config key.
Only flags that the application maps into `CliOverrides` affect config values.

Flatten it into an application command enum:

1. Keep the application's own `Parser` type.
2. Keep the application's own `Subcommand` enum.
3. Add `#[command(flatten)] Config(ConfigCommand)` to that enum.
4. Clap expands the flattened `ConfigCommand` variants into the same command
   level as the application's own variants.
5. Match the `Config(command)` variant and pass it to `handle_config_command`.

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

If no output path is provided, the command writes `config.example.yaml` in the
current directory. Add `--schema schemas/myapp.schema.json` to bind generated
TOML and YAML templates to generated JSON Schemas. Split YAML templates bind the
matching section schema. The command also writes the root and section schemas to
the selected schema path.

```bash
demo config-template --output config.example.toml --schema schemas/myapp.schema.json
```

Generate root and section JSON Schemas:

```bash
demo config-schema --output schemas/myapp.schema.json
```

Validate the complete runtime config tree:

```bash
demo config-validate
```

Generated editor schemas intentionally avoid required-field diagnostics for
split files. `config-validate` loads includes, applies defaults, and runs final
`confique` validation. It prints `Configuration is ok` when validation
succeeds.

## Shell Completions

Print completions to stdout:

```bash
demo completions zsh
```

Install completions:

```bash
demo install-completions zsh
```

The installer supports Bash, Elvish, Fish, PowerShell, and Zsh. It writes the
completion file under the user's home directory and updates the shell startup
file for shells that require it.
