# Konfigurationsschema

[English](../en/schema.html) | [中文](../zh/schema.html) | [日本語](../ja/schema.html) | [한국어](../ko/schema.html) | [Français](../fr/schema.html) | [Deutsch](schema.html) | [Español](../es/schema.html) | [Português](../pt/schema.html) | [Svenska](../sv/schema.html) | [Suomi](../fi/schema.html) | [Nederlands](../nl/schema.html)

Anwendungsschemas sind normale `confique`-Konfigurationstypen. Das Root-Schema
muss `ConfigSchema` implementieren, damit `rust-config-tree` rekursive Includes
aus der zwischengeschalteten `confique`-Schicht entdecken kann.

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
Abschnitte werden immer fuer das Laden zur Laufzeit genutzt. Fuege
`#[schemars(extend("x-tree-split" = true))]` hinzu, wenn ein nested Feld
zusaetzlich als eigenes `config/*.yaml`-Template und `schemas/*.schema.json`-Schema
erzeugt werden soll:

```rust
#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    server: ServerConfig,
}
```

Die natuerliche YAML-Form ist:

```yaml
server:
  bind: 127.0.0.1
  port: 8080
```

## Nur-Umgebung-Felder

Markiere ein Blattfeld mit `#[schemars(extend("x-env-only" = true))]`, wenn sein Wert nur aus einer Umgebungsvariable kommen soll und nicht in generierten Konfigurationsdateien erscheinen darf. Generierte YAML-Vorlagen und JSON-Schemas lassen env-only-Felder weg, und dadurch leere Elternobjekte werden entfernt.

```rust
#[config(env = "APP_SECRET")]
#[schemars(extend("x-env-only" = true))]
secret: String,
```

## Abschnittspfade fuer Vorlagen ueberschreiben

Wenn eine Vorlagenquelle keine Includes hat, kann die Crate Kind-Vorlagendateien
aus mit `x-tree-split` markierten verschachtelten Schemaabschnitten ableiten. Der Standardpfad auf oberster
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
