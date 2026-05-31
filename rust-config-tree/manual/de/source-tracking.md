# Quellenverfolgung

[English](../en/source-tracking.html) | [中文](../zh/source-tracking.html) | [日本語](../ja/source-tracking.html) | [한국어](../ko/source-tracking.html) | [Français](../fr/source-tracking.html) | [Deutsch](source-tracking.html) | [Español](../es/source-tracking.html) | [Português](../pt/source-tracking.html) | [Svenska](../sv/source-tracking.html) | [Suomi](../fi/source-tracking.html) | [Nederlands](../nl/source-tracking.html)

Verwende `load_config_with_figment`, um den Figment-Graphen zu behalten, der
beim Laufzeitladen verwendet wurde:

```rust
use rust_config_tree::load_config_with_figment;

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Der zurueckgegebene Figment-Wert kann Quellenfragen fuer Laufzeitwerte
beantworten:

```rust
if let Some(metadata) = figment.find_metadata("database.pool_size") {
    let source = metadata.interpolate(
        &figment::Profile::Default,
        &["database", "pool_size"],
    );

    println!("database.pool_size came from {source}");
}
```

Fuer Werte, die von `ConfiqueEnvProvider` geliefert werden, gibt die
Interpolation den nativen Umgebungsvariablennamen zurueck, der im Schema
deklariert ist:

```text
database.pool_size came from APP_DATABASE_POOL_SIZE
```

## TRACE-Ereignisse

Der Loader gibt Quellenverfolgungsereignisse mit `tracing::trace!` aus. Das
geschieht nur, wenn TRACE aktiviert ist:

```rust
use rust_config_tree::{load_config_with_figment, trace_config_sources};

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;

// If the tracing subscriber is initialized after config loading, emit the
// same source events again after installing the subscriber.
trace_config_sources::<AppConfig>(&figment);
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Jedes Ereignis verwendet das Ziel `rust_config_tree::config` und enthaelt:

- `config_key`: den gepunkteten Konfigurationsschluessel.
- `source`: die gerenderte Quellenmetadatenangabe.

Werte, die nur aus `confique`-Defaults stammen, haben keine
Figment-Laufzeitmetadaten. Sie werden als
`confique default or unset optional field` gemeldet.
