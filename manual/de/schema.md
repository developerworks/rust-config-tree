# Konfigurationsschema

[English](../en/schema.html) | [中文](../zh/schema.html) | [日本語](../ja/schema.html) | [한국어](../ko/schema.html) | [Français](../fr/schema.html) | [Deutsch](schema.html) | [Español](../es/schema.html) | [Português](../pt/schema.html) | [Svenska](../sv/schema.html) | [Suomi](../fi/schema.html) | [Nederlands](../nl/schema.html)

Anwendungsschemas sind normale `confique`-Konfigurationstypen. Das Root-Schema
muss `ConfigSchema` implementieren, damit `rust-config-tree` rekursive Includes
aus der zwischengeschalteten `confique`-Schicht entdecken kann.

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

## Include-Feld

Das Include-Feld kann beliebig heissen. `rust-config-tree` kennt es nur ueber
`ConfigSchema::include_paths`.

Das Feld sollte normalerweise einen leeren Default haben:

```rust
#[config(default = [])]
include: Vec<PathBuf>,
```

Der Loader erhaelt fuer jede Datei eine teilweise geladene Schicht. Dadurch
kann er Kind-Konfigurationsdateien entdecken, bevor das finale Schema
zusammengefuehrt und validiert wird.

## Verschachtelte Abschnitte

Verwende `#[config(nested)]` fuer strukturierte Abschnitte. Verschachtelte
Abschnitte sind sowohl fuer das Laden zur Laufzeit als auch fuer die
Vorlagenaufteilung wichtig:

```rust
#[derive(Debug, Config)]
struct AppConfig {
    #[config(nested)]
    server: ServerConfig,
}
```

Die natuerliche YAML-Form ist:

```yaml
server:
  bind: 127.0.0.1
  port: 8080
```

## Abschnittspfade fuer Vorlagen ueberschreiben

Wenn eine Vorlagenquelle keine Includes hat, kann die Crate Kind-Vorlagendateien
aus verschachtelten Schemaabschnitten ableiten. Der Standardpfad auf oberster
Ebene ist `config/<section>.yaml`.

Ueberschreibe diesen Pfad mit `template_path_for_section`:

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
