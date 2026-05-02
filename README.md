# rust-config-tree

`rust-config-tree` provides configuration-tree loading and CLI helpers for Rust
applications that use layered config files.

It handles:

- loading a `confique` schema into a directly usable config object
- `config-template`, `completions`, and `install-completions` command handlers
- config template generation for YAML, TOML, JSON, and JSON5
- recursive include traversal
- relative include paths resolved from the file declaring them
- lexical path normalization
- include cycle detection
- deterministic traversal order
- mirrored template target collection

Applications provide their schema by deriving `confique::Config` and
implementing `ConfigSchema` to expose the schema's include field.

## Example

```rust
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use confique::Config;
use rust_config_tree::{ConfigCommand, ConfigSchema, handle_config_command, load_config};

#[derive(Debug, Config)]
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

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT license

at your option.
