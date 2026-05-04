# Configuration Schema

[English](schema.html) | [中文](../zh/schema.html) | [日本語](../ja/schema.html) | [한국어](../ko/schema.html) | [Français](../fr/schema.html) | [Deutsch](../de/schema.html) | [Español](../es/schema.html) | [Português](../pt/schema.html) | [Svenska](../sv/schema.html) | [Suomi](../fi/schema.html) | [Nederlands](../nl/schema.html)

Application schemas are normal `confique` config types. The root schema must
implement `ConfigSchema` so `rust-config-tree` can discover recursive includes
from the intermediate `confique` layer.

```rust
use std::path::PathBuf;

use confique::Config;
use schemars::JsonSchema;
use rust_config_tree::ConfigSchema;

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    database: DatabaseConfig,
}

#[derive(Debug, Config, JsonSchema)]
struct DatabaseConfig {
    #[config(env = "APP_DATABASE_URL")]
    url: String,

    #[config(default = 16)]
    #[config(env = "APP_DATABASE_POOL_SIZE")]
    pool_size: u32,
}

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}
```

## Include Field

The include field can have any name. `rust-config-tree` only knows about it
through `ConfigSchema::include_paths`.

The field should normally have an empty default:

```rust
#[config(default = [])]
include: Vec<PathBuf>,
```

The loader receives a partially loaded layer for each file. That lets it
discover child config files before the final schema is merged and validated.

## Nested Sections

Use `#[config(nested)]` for structured sections. Nested sections are always
used for runtime loading. Add `#[schemars(extend("x-tree-split" = true))]`
when a nested field should also be generated as an independent
`config/*.yaml` template and `schemas/*.schema.json` schema:

```rust
#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    server: ServerConfig,
}
```

The natural YAML shape is:

```yaml
server:
  bind: 127.0.0.1
  port: 8080
```

## Environment-Only Fields

Mark a leaf field with `#[schemars(extend("x-env-only" = true))]` when the value must be supplied only by an environment variable and should not appear in generated config files. Generated YAML templates and JSON Schemas omit env-only fields, and empty parent objects left behind by those omissions are pruned.

```rust
#[config(env = "APP_SECRET")]
#[schemars(extend("x-env-only" = true))]
secret: String,
```

## Field Value Validation

Generated `*.schema.json` files are for IDE completion and basic editor checks
only. They do not decide whether a concrete field value is legal for the
application.

Implement field value validation in code with
`#[config(validate = Self::validate)]`. The validator runs when the final config
is loaded through `load_config` or checked through `config-validate`.

## Template Section Overrides

When a template source has no includes, the crate can derive child template
files from nested schema sections marked with `x-tree-split`. The default
top-level path is `config/<section>.yaml`.

Override that path with `template_path_for_section`:

```rust
impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }

    fn template_path_for_section(section_path: &[&str]) -> Option<PathBuf> {
        match section_path {
            ["database"] => Some(PathBuf::from("examples/database.yaml")),
            _ => None,
        }
    }
}
```
