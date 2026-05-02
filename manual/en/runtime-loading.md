# Runtime Loading

[English](runtime-loading.html) | [中文](../zh/runtime-loading.html)

Runtime loading is intentionally split between Figment and confique:

```text
figment:
  runtime file loading
  runtime environment loading
  runtime source metadata

confique:
  schema metadata
  defaults
  validation
  config templates
```

The main API is:

```rust
use rust_config_tree::load_config;

let config = load_config::<AppConfig>("config.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Use `load_config_with_figment` when the application needs source metadata:

```rust
use rust_config_tree::load_config_with_figment;

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;
# let _ = (config, figment);
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

## Loading Steps

The high-level loader performs these steps:

1. Resolve the root config path lexically.
2. Load the first `.env` file found by walking upward from the root config
   directory.
3. Load each config file as a partial layer to discover includes.
4. Build a Figment graph from the discovered config files.
5. Merge the `ConfiqueEnvProvider` with higher priority than files.
6. Optionally merge application-specific CLI overrides.
7. Extract a `confique` layer from Figment.
8. Apply `confique` code defaults.
9. Validate and construct the final schema.

`load_config` and `load_config_with_figment` perform steps 1-5 and 7-9.
Step 6 is application-specific because this crate cannot infer how a CLI flag
maps to a schema field.

## File Formats

The runtime file provider is selected from the config path extension:

- `.yaml` and `.yml` use YAML.
- `.toml` uses TOML.
- `.json` and `.json5` use JSON.
- unknown or missing extensions use YAML.

Template generation still uses confique's template renderers for YAML, TOML,
and JSON5-compatible output.

## Include Priority

The high-level loader merges file providers so included files are lower priority
than the file that included them. The root config file has the highest file
priority.

Environment variables have higher priority than all config files. `confique`
defaults are only used for values that are not supplied by runtime providers.

When CLI overrides are merged after `build_config_figment`, the full precedence
is:

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```

The command-line syntax is not defined by `rust-config-tree`. A flag like
`--server-port` can override `server.port` if the application maps that parsed
value into a nested serialized provider. A dotted `--server.port` or `a.b.c`
syntax only exists if the application implements it.

```rust
use figment::providers::Serialized;
use serde::Serialize;
use rust_config_tree::{build_config_figment, load_config_from_figment};

#[derive(Debug, Serialize)]
struct CliOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    server: Option<CliServerOverrides>,
}

#[derive(Debug, Serialize)]
struct CliServerOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    port: Option<u16>,
}

let cli_overrides = CliOverrides {
    server: Some(CliServerOverrides { port: Some(9000) }),
};

let figment = build_config_figment::<AppConfig>("config.yaml")?
    .merge(Serialized::defaults(cli_overrides));

let config = load_config_from_figment::<AppConfig>(&figment)?;
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```
