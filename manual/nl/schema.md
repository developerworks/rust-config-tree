# Configuratieschema

[English](../en/schema.html) | [中文](../zh/schema.html) | [日本語](../ja/schema.html) | [한국어](../ko/schema.html) | [Français](../fr/schema.html) | [Deutsch](../de/schema.html) | [Español](../es/schema.html) | [Português](../pt/schema.html) | [Svenska](../sv/schema.html) | [Suomi](../fi/schema.html) | [Nederlands](schema.html)

Toepassingsschema's zijn normale `confique`-configuratietypen. Het rootschema
moet `ConfigSchema` implementeren zodat `rust-config-tree` recursieve includes
uit de tussenliggende `confique`-laag kan ontdekken.

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

Gebruik `#[config(nested)]` voor gestructureerde secties. Geneste secties
worden altijd gebruikt voor runtime laden. Voeg
`#[schemars(extend("x-tree-split" = true))]` toe wanneer een genest veld ook
als eigen `*.yaml`-sjabloon en `<section>.schema.json`-schema moet
worden gegenereerd:

```rust
#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    server: ServerConfig,
}
```

De natuurlijke YAML-vorm is:

```yaml
server:
  bind: 127.0.0.1
  port: 8080
```

## Omgevings-only velden

Markeer een leafveld met `#[schemars(extend("x-env-only" = true))]` wanneer de waarde alleen uit een omgevingsvariabele mag komen en niet in gegenereerde configuratiebestanden mag verschijnen. Gegenereerde YAML-sjablonen en JSON Schemas laten env-only velden weg, en lege bovenliggende objecten die daardoor overblijven worden verwijderd.

```rust
#[config(env = "APP_SECRET")]
#[schemars(extend("x-env-only" = true))]
secret: String,
```

## Veldwaardevalidatie

Gegenereerde `*.schema.json`-bestanden zijn alleen voor IDE-completion en basale
editorcontroles. Ze bepalen niet of een concrete veldwaarde geldig is voor de
toepassing.

Veldwaardevalidatie moet in code worden geimplementeerd met
`#[config(validate = Self::validate)]`. De validator draait wanneer de
uiteindelijke configuratie wordt geladen met `load_config` of gecontroleerd met
`config-validate`.

## Overrides voor sjabloonsecties

Wanneer een sjabloonbron geen includes heeft, kan de crate kind-
sjabloonbestanden afleiden uit geneste schemaselecties gemarkeerd met `x-tree-split`. Het standaardpad op het
hoogste niveau is `<section>.yaml`.

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
