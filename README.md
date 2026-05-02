# rust-config-tree

`rust-config-tree` provides configuration-tree loading and CLI helpers for Rust
applications that use layered config files.

It handles:

- loading a `confique` schema into a directly usable config object
- `config-template`, `completions`, and `install-completions` command handlers
- config template generation for YAML, TOML, JSON, and JSON5
- recursive include traversal
- `.env` loading before environment values are merged
- relative include paths resolved from the file declaring them
- lexical path normalization
- include cycle detection
- deterministic traversal order
- mirrored template target collection
- automatic YAML template splitting for nested schema sections

Applications provide their schema by deriving `confique::Config` and
implementing `ConfigSchema` to expose the schema's include field.

## Install

```toml
[dependencies]
rust-config-tree = "0.1"
confique = { version = "0.4", features = ["yaml", "toml", "json5"] }
clap = { version = "4", features = ["derive"] }
```

## Configuration Schema

Your application schema owns the include field. `rust-config-tree` only needs a
small adapter that extracts includes from the intermediate `confique` layer.

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

Relative include paths are resolved from the file that declares them:

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

Load the final schema with `load_config`:

```rust
use rust_config_tree::load_config;

# use std::path::PathBuf;
# use confique::Config;
# use rust_config_tree::ConfigSchema;
# #[derive(Debug, Config)]
# struct AppConfig {
#     #[config(default = [])]
#     include: Vec<PathBuf>,
#     #[config(default = "paper")]
#     mode: String,
# }
# impl ConfigSchema for AppConfig {
#     fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
#         layer.include.clone().unwrap_or_default()
#     }
# }
let config = load_config::<AppConfig>("config.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

`load_config` loads the first `.env` file found by walking upward from the root
config file's directory before asking `confique` to read environment variables.
Values already present in the process environment are preserved and take
precedence over `.env` values.

## Template Generation

Templates are rendered with the same schema and include traversal rules. The
output format is inferred from the output path:

- `.yaml` and `.yml` generate YAML
- `.toml` generates TOML
- `.json` and `.json5` generate JSON5-compatible templates
- unknown or missing extensions generate YAML

Use `write_config_templates` to create a root template and every template file
reachable from its include tree:

```rust
use rust_config_tree::write_config_templates;

# use std::path::PathBuf;
# use confique::Config;
# use rust_config_tree::ConfigSchema;
# #[derive(Debug, Config)]
# struct AppConfig {
#     #[config(default = [])]
#     include: Vec<PathBuf>,
# }
# impl ConfigSchema for AppConfig {
#     fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
#         layer.include.clone().unwrap_or_default()
#     }
# }
write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Template generation chooses its source tree in this order:

- an existing config path
- an existing output template path
- the output path, treated as a new empty template tree

If a source node has no include list, `rust-config-tree` derives child template
files from nested `confique` sections. With the schema above, an empty
`config.example.yaml` source produces:

```text
config.example.yaml
config/server.yaml
```

The root template receives an include block for `config/server.yaml`. YAML
targets that map to a nested section, such as `config/server.yaml`, contain only
that section. Further nested sections are split recursively in the same way.

Override `template_path_for_section` when a section should be generated at a
different path:

```rust
# use std::path::PathBuf;
# use confique::Config;
# use rust_config_tree::ConfigSchema;
# #[derive(Debug, Config)]
# struct AppConfig {
#     #[config(default = [])]
#     include: Vec<PathBuf>,
#     #[config(nested)]
#     server: ServerConfig,
# }
# #[derive(Debug, Config)]
# struct ServerConfig {
#     #[config(default = 8080)]
#     port: u16,
# }
impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }

    fn template_path_for_section(section_path: &[&str]) -> Option<PathBuf> {
        match section_path {
            ["server"] => Some(PathBuf::from("examples/server.yaml")),
            _ => None,
        }
    }
}
```

The default section path is `config/<section>.yaml` for top-level nested
sections. Nested children are placed under their parent file stem, for example
`config/trading/risk.yaml`.

## CLI Integration

Flatten `ConfigCommand` into your existing clap command enum to add:

- `config-template`
- `completions`
- `install-completions`

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

`config-template --output <path>` writes templates to the selected path. If no
output path is provided, it writes `config.example.yaml` in the current
directory.

`completions <shell>` prints completions to stdout.

`install-completions <shell>` writes completions under the user's home
directory and updates the shell startup file when the shell requires it. Bash,
Elvish, Fish, PowerShell, and Zsh are supported.

## Lower-Level Tree API

Use `load_config_tree` when you do not use `confique` or when you need direct
access to traversal results:

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

let tree = load_config_tree("config.yaml", load_source)?;
for node in tree.nodes() {
    println!("{}", node.path().display());
}
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

The tree API normalizes paths lexically, rejects empty include paths, detects
recursive include cycles, and skips files that were already loaded through
another include branch.

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT license

at your option.
