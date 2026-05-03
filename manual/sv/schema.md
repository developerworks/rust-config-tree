# Konfigurationsschema

[English](../en/schema.html) | [中文](../zh/schema.html) | [日本語](../ja/schema.html) | [한국어](../ko/schema.html) | [Français](../fr/schema.html) | [Deutsch](../de/schema.html) | [Español](../es/schema.html) | [Português](../pt/schema.html) | [Svenska](schema.html) | [Suomi](../fi/schema.html) | [Nederlands](../nl/schema.html)

Programscheman ar vanliga `confique`-konfigurationstyper. Rotschemat maste
implementera `ConfigSchema` sa `rust-config-tree` kan upptacka rekursiva
includes fran det mellanliggande `confique`-lagret.

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

## Include-falt

Include-faltet kan ha vilket namn som helst. `rust-config-tree` kanner bara till
det via `ConfigSchema::include_paths`.

Faltet bor normalt ha en tom standard:

```rust
#[config(default = [])]
include: Vec<PathBuf>,
```

Laddaren tar emot ett partiellt laddat lager for varje fil. Det gor att den kan
upptacka barnkonfigurationsfiler innan det slutliga schemat slas samman och
valideras.

## Nastlade sektioner

Anvand `#[config(nested)]` for strukturerade sektioner. Nastlade sektioner
anvands alltid for runtime-laddning. Lagg till
`#[schemars(extend("x-tree-split" = true))]` nar ett nastlat falt ocksa ska
genereras som egen `config/*.yaml`-mall och `schemas/*.schema.json`-schema:

```rust
#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    server: ServerConfig,
}
```

Den naturliga YAML-formen ar:

```yaml
server:
  bind: 127.0.0.1
  port: 8080
```

## Overstyrning av mallsektioner

Nar en mallkalla saknar includes kan craten harleda barnmallfiler fran nastlade
schemasektioner markerade med `x-tree-split`. Standardsokvagen pa toppniva ar `config/<section>.yaml`.

Overstyr den sokvagen med `template_path_for_section`:

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
