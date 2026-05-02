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
6. Extract a `confique` layer from Figment.
7. Apply `confique` code defaults.
8. Validate and construct the final schema.

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
