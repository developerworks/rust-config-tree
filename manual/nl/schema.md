# Configuratieschema

[English](../en/schema.html) | [中文](../zh/schema.html) | [日本語](../ja/schema.html) | [한국어](../ko/schema.html) | [Français](../fr/schema.html) | [Deutsch](../de/schema.html) | [Español](../es/schema.html) | [Português](../pt/schema.html) | [Svenska](../sv/schema.html) | [Suomi](../fi/schema.html) | [Nederlands](schema.html)

Toepassingsschema's zijn normale `confique`-configuratietypen. Het rootschema
moet `ConfigSchema` implementeren zodat `rust-config-tree` recursieve includes
uit de tussenliggende `confique`-laag kan ontdekken.

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

## Include-veld

Het include-veld mag elke naam hebben. `rust-config-tree` kent het alleen via
`ConfigSchema::include_paths`.

Het veld heeft normaal een lege default:

```rust
#[config(default = [])]
include: Vec<PathBuf>,
```

De loader ontvangt voor elk bestand een gedeeltelijk geladen laag. Daardoor kan
hij kindconfiguratiebestanden ontdekken voordat het uiteindelijke schema wordt
samengevoegd en gevalideerd.

## Geneste secties

Gebruik `#[config(nested)]` voor gestructureerde secties. Geneste secties zijn
belangrijk voor zowel runtime laden als sjabloonsplitsing:

```rust
#[derive(Debug, Config)]
struct AppConfig {
    #[config(nested)]
    server: ServerConfig,
}
```

De natuurlijke YAML-vorm is:

```yaml
server:
  bind: 127.0.0.1
  port: 8080
```

## Overrides voor sjabloonsecties

Wanneer een sjabloonbron geen includes heeft, kan de crate kind-
sjabloonbestanden afleiden uit geneste schemaselecties. Het standaardpad op het
hoogste niveau is `config/<section>.yaml`.

Overschrijf dat pad met `template_path_for_section`:

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
