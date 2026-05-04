# Schema de configuration

[English](../en/schema.html) | [中文](../zh/schema.html) | [日本語](../ja/schema.html) | [한국어](../ko/schema.html) | [Français](schema.html) | [Deutsch](../de/schema.html) | [Español](../es/schema.html) | [Português](../pt/schema.html) | [Svenska](../sv/schema.html) | [Suomi](../fi/schema.html) | [Nederlands](../nl/schema.html)

Les schemas d'application sont des types de configuration `confique` normaux.
Le schema racine doit implementer `ConfigSchema` afin que `rust-config-tree`
puisse decouvrir les inclusions recursives depuis la couche intermediaire
`confique`.

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

## Champ d'inclusion

Le champ d'inclusion peut avoir n'importe quel nom. `rust-config-tree` ne le
connait qu'au travers de `ConfigSchema::include_paths`.

Le champ doit normalement avoir une valeur par defaut vide :

```rust
#[config(default = [])]
include: Vec<PathBuf>,
```

Le chargeur recoit une couche partiellement chargee pour chaque fichier. Cela
lui permet de decouvrir les fichiers de configuration enfants avant que le
schema final soit fusionne et valide.

## Sections imbriquees

Utilisez `#[config(nested)]` pour les sections structurees. Les sections
imbriquees sont toujours utilisees pour le chargement d'execution. Ajoutez
`#[schemars(extend("x-tree-split" = true))]` lorsqu'un champ imbrique doit aussi
etre genere comme son propre modele `config/*.yaml` et schema
`schemas/*.schema.json` :

```rust
#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    server: ServerConfig,
}
```

La forme YAML naturelle est :

```yaml
server:
  bind: 127.0.0.1
  port: 8080
```

## Champs reserves aux variables d environnement

Marquez un champ feuille avec `#[schemars(extend("x-env-only" = true))]` lorsque sa valeur doit venir uniquement d une variable d environnement et ne doit pas apparaitre dans les fichiers de configuration generes. Les modeles YAML et schemas JSON generes omettent les champs env-only, et les objets parents devenus vides sont supprimes.

```rust
#[config(env = "APP_SECRET")]
#[schemars(extend("x-env-only" = true))]
secret: String,
```

## Validation des valeurs de champ

Les fichiers `*.schema.json` generes servent uniquement a la completion IDE et
aux controles d'editeur de base. Ils ne decident pas si une valeur de champ
concrete est valide pour l'application.

La validation des valeurs doit etre implementee dans le code avec
`#[config(validate = Self::validate)]`. Ce validateur s'execute quand la
configuration finale est chargee par `load_config` ou verifiee par
`config-validate`.

## Remplacements de chemin de section pour les modeles

Lorsqu'une source de modele n'a pas d'inclusions, la crate peut deriver les
fichiers modeles enfants depuis les sections de schema imbriquees marquees `x-tree-split`. Le chemin de
premier niveau par defaut est `config/<section>.yaml`.

Remplacez ce chemin avec `template_path_for_section` :

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
