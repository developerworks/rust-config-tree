# Konfigurationsschema

[English](../en/schema.html) | [中文](../zh/schema.html) | [日本語](../ja/schema.html) | [한국어](../ko/schema.html) | [Français](../fr/schema.html) | [Deutsch](../de/schema.html) | [Español](../es/schema.html) | [Português](../pt/schema.html) | [Svenska](schema.html) | [Suomi](../fi/schema.html) | [Nederlands](../nl/schema.html)

Programscheman ar vanliga `confique`-konfigurationstyper. Rotschemat maste
implementera `ConfigSchema` sa `rust-config-tree` kan upptacka rekursiva
includes fran det mellanliggande `confique`-lagret.

```rust
use std::path::PathBuf;

use confique::Config;
use rust_config_tree::ConfigSchema;

#[derive(Debug, Config)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(nested)]
    database: DatabaseConfig,
}

#[derive(Debug, Config)]
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

Anvand `#[config(nested)]` for strukturerade sektioner. Nastlade sektioner ar
viktiga for bade runtime-laddning och malluppdelning:

```rust
#[derive(Debug, Config)]
struct AppConfig {
    #[config(nested)]
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
schemasektioner. Standardsokvagen pa toppniva ar `config/<section>.yaml`.

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
