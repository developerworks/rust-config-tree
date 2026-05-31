# Brontracking

[English](../en/source-tracking.html) | [中文](../zh/source-tracking.html) | [日本語](../ja/source-tracking.html) | [한국어](../ko/source-tracking.html) | [Français](../fr/source-tracking.html) | [Deutsch](../de/source-tracking.html) | [Español](../es/source-tracking.html) | [Português](../pt/source-tracking.html) | [Svenska](../sv/source-tracking.html) | [Suomi](../fi/source-tracking.html) | [Nederlands](source-tracking.html)

Gebruik `load_config_with_figment` om de Figment-grafiek te behouden die door
runtime laden is gebruikt:

```rust
use rust_config_tree::load_config_with_figment;

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

De geretourneerde Figment-waarde kan bronvragen voor runtimewaarden beantwoorden:

```rust
if let Some(metadata) = figment.find_metadata("database.pool_size") {
    let source = metadata.interpolate(
        &figment::Profile::Default,
        &["database", "pool_size"],
    );

    println!("database.pool_size came from {source}");
}
```

Voor waarden die door `ConfiqueEnvProvider` worden geleverd, retourneert
interpolatie de native naam van de omgevingsvariabele die in het schema is
gedeclareerd:

```text
database.pool_size came from APP_DATABASE_POOL_SIZE
```

## TRACE-events

De loader emitteert brontrackingevents met `tracing::trace!`. Dit gebeurt
alleen wanneer TRACE is ingeschakeld:

```rust
use rust_config_tree::{load_config_with_figment, trace_config_sources};

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;

// If the tracing subscriber is initialized after config loading, emit the
// same source events again after installing the subscriber.
trace_config_sources::<AppConfig>(&figment);
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Elk event gebruikt het target `rust_config_tree::config` en bevat:

- `config_key`: de gestippelde configuratiesleutel.
- `source`: de gerenderde bronmetadata.

Waarden die alleen uit `confique`-defaults komen, hebben geen Figment-
runtime-metadata. Ze worden gerapporteerd als `confique default or unset optional field`.
