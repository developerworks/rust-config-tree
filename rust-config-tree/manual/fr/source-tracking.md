# Suivi des sources

[English](../en/source-tracking.html) | [中文](../zh/source-tracking.html) | [日本語](../ja/source-tracking.html) | [한국어](../ko/source-tracking.html) | [Français](source-tracking.html) | [Deutsch](../de/source-tracking.html) | [Español](../es/source-tracking.html) | [Português](../pt/source-tracking.html) | [Svenska](../sv/source-tracking.html) | [Suomi](../fi/source-tracking.html) | [Nederlands](../nl/source-tracking.html)

Utilisez `load_config_with_figment` pour conserver le graphe Figment utilise par
le chargement d'execution :

```rust
use rust_config_tree::load_config_with_figment;

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

La valeur Figment renvoyee peut repondre aux questions de source pour les
valeurs d'execution :

```rust
if let Some(metadata) = figment.find_metadata("database.pool_size") {
    let source = metadata.interpolate(
        &figment::Profile::Default,
        &["database", "pool_size"],
    );

    println!("database.pool_size came from {source}");
}
```

Pour les valeurs fournies par `ConfiqueEnvProvider`, l'interpolation renvoie le
nom natif de la variable d'environnement declaree dans le schema :

```text
database.pool_size came from APP_DATABASE_POOL_SIZE
```

## Evenements TRACE

Le chargeur emet des evenements de suivi des sources avec `tracing::trace!`. Il
ne le fait que lorsque TRACE est active :

```rust
use rust_config_tree::{load_config_with_figment, trace_config_sources};

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;

// If the tracing subscriber is initialized after config loading, emit the
// same source events again after installing the subscriber.
trace_config_sources::<AppConfig>(&figment);
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Chaque evenement utilise la cible `rust_config_tree::config` et inclut :

- `config_key` : la cle de configuration avec points.
- `source` : les metadonnees de source rendues.

Les valeurs qui viennent uniquement des valeurs par defaut `confique` n'ont pas
de metadonnees Figment d'execution. Elles sont signalees comme
`confique default or unset optional field`.

