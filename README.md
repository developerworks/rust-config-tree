# rust-config-tree

[English](README.md) | [中文](README.zh.md)

`rust-config-tree` provides configuration-tree loading and CLI helpers for Rust
applications that use layered config files.

Project manual: <https://developerworks.github.io/rust-config-tree/>. English
and Chinese manuals are published as independent mdBook sites with language
switch links.

It handles:

- loading a `confique` schema into a directly usable config object through
  Figment runtime providers
- `config-template`, `completions`, and `install-completions` command handlers
- Draft 7 root and section JSON Schema generation for editor completion and validation
- config template generation for YAML, TOML, JSON, and JSON5
- schema directives for TOML and YAML templates without adding runtime fields
- recursive include traversal
- `.env` loading before environment values are merged
- source tracking through Figment metadata
- TRACE-level source tracking logs through `tracing`
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
figment = { version = "0.10", features = ["yaml", "env"] }
schemars = { version = "1", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
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

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = load_config::<AppConfig>("config.yaml")?;
    println!("{config:#?}");

    Ok(())
}
```

`load_config` loads the first `.env` file found by walking upward from the root
config file's directory before asking Figment to read schema-declared
environment variables. Values already present in the process environment are
preserved and take precedence over `.env` values.

Runtime config loading is performed through Figment. `confique` remains
responsible for schema metadata, defaults, validation, and template generation.
Environment variable names are read from `#[config(env = "...")]`; the loader
does not use `Env::split("_")` or `Env::split("__")`, so a variable such as
`APP_DATABASE_POOL_SIZE` can map to a field named `database.pool_size`.

`load_config` does not read command-line arguments because CLI flags are
application-specific. Add CLI overrides by merging a provider after
`build_config_figment`, then validate with `load_config_from_figment`:

CLI flag names are not derived from config paths. Use normal application flags
such as `--server-port` or `--database-url`; do not rely on `--server.port` or
`a.b.c` unless the application deliberately implements that parser. The nested
serialized override shape decides which config key is overridden.

Only values represented in the application's `CliOverrides` provider can
override configuration. This is intended for frequently adjusted runtime
parameters, where changing a flag for one run is better than editing a config
file. Keep stable configuration in files and expose only deliberate temporary
overrides as CLI flags.

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

With CLI overrides merged this way, runtime precedence is:

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```

Use `load_config_with_figment` when the caller needs source metadata:

```rust
use rust_config_tree::load_config_with_figment;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;

    if let Some(metadata) = figment.find_metadata("mode") {
        let source = metadata.interpolate(&figment::Profile::Default, &["mode"]);
        println!("mode came from {source}");
    }

    println!("{config:#?}");

    Ok(())
}
```

The loader also emits config source tracking with `tracing::trace!`. Those
events are produced only when TRACE is enabled by the application's tracing
subscriber. If tracing is initialized after config loading, call
`trace_config_sources::<AppConfig>(&figment)` after installing the subscriber.

## Template Generation

Templates are rendered with the same schema and include traversal rules. The
output format is inferred from the output path:

- `.yaml` and `.yml` generate YAML
- `.toml` generates TOML
- `.json` and `.json5` generate JSON5-compatible templates
- unknown or missing extensions generate YAML

Use `write_config_schemas` to create Draft 7 JSON Schemas for the root config
and nested sections. The generated schemas omit `required` constraints so IDEs
can offer completion for partial config files without reporting missing fields:

```rust
use rust_config_tree::write_config_schemas;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;

    Ok(())
}
```

For a schema with `server` and `log` sections, this writes
`schemas/myapp.schema.json`, `schemas/server.schema.json`, and
`schemas/log.schema.json`. The root schema contains only fields that belong in
the root config file, such as `include` and root scalar fields. It intentionally
omits nested section properties, so `server` and `log` are completed only when
editing their own section YAML files.

Use `write_config_templates` to create a root template and every template file
reachable from its include tree:

```rust
use rust_config_tree::write_config_templates;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;

    Ok(())
}
```

Use `write_config_templates_with_schema` when generated TOML and YAML templates
should bind those schemas for IDE completion and validation:

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

Root TOML/YAML targets bind the root schema and do not complete child section
fields. Split section YAML targets bind their matching section schema, for
example `config/log.yaml` receives
`# yaml-language-server: $schema=../schemas/log.schema.json`. JSON and JSON5
targets intentionally do not receive a `$schema` field; bind them with editor
settings such as VS Code `json.schemas`.

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
use std::path::PathBuf;

use confique::Config;
use rust_config_tree::ConfigSchema;

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
- `config-schema`
- `config-validate`
- `completions`
- `install-completions`

The consuming application keeps its own `Parser` type and its own command enum.
`rust-config-tree` only contributes reusable subcommands:

1. Add `#[command(subcommand)] command: Command` to the application's parser.
2. Add `#[command(flatten)] Config(ConfigCommand)` to the application's
   `Subcommand` enum.
3. Clap expands the flattened variants into the same subcommand level as the
   application's own commands.
4. Match that variant and call `handle_config_command::<Cli, AppConfig>`.

Application-specific config override flags stay on the application's own parser.
For example, `--server-port` can map to `server.port` by building a nested
`CliOverrides { server: Some(CliServerOverrides { port }) }` value and merging
it with `Serialized::defaults`.

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

`config-template --output <path>` writes templates to the selected path. If no
output path is provided, it writes `config.example.yaml` in the current
directory. Add `--schema <path>` to bind TOML and YAML templates to a generated
JSON Schema set without adding a runtime `$schema` field. This also writes the
root schema and section schemas to the selected schema path.

`config-schema --output <path>` writes the root Draft 7 JSON Schema and
section schemas. If no output path is provided, the root schema is written to
`schemas/config.schema.json`.

`config-validate` loads the full runtime config tree and runs `confique`
defaults and validation. Use editor schemas for non-noisy completion while
editing split files; use this command for required fields and final config
validation. It prints `Configuration is ok` when validation succeeds.

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

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let tree = load_config_tree("config.yaml", load_source)?;

    for node in tree.nodes() {
        println!("{}", node.path().display());
    }

    Ok(())
}
```

The tree API normalizes paths lexically, rejects empty include paths, detects
recursive include cycles, and skips files that were already loaded through
another include branch.

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT license

at your option.
