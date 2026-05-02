# Seguimiento de origen

[English](../en/source-tracking.html) | [中文](../zh/source-tracking.html) | [日本語](../ja/source-tracking.html) | [한국어](../ko/source-tracking.html) | [Français](../fr/source-tracking.html) | [Deutsch](../de/source-tracking.html) | [Español](source-tracking.html) | [Português](../pt/source-tracking.html) | [Svenska](../sv/source-tracking.html) | [Suomi](../fi/source-tracking.html) | [Nederlands](../nl/source-tracking.html)

Usa `load_config_with_figment` para conservar el grafo Figment usado por la
carga en tiempo de ejecución:

```rust
use rust_config_tree::load_config_with_figment;

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

El valor Figment devuelto puede responder preguntas de origen para valores en
tiempo de ejecución:

```rust
if let Some(metadata) = figment.find_metadata("database.pool_size") {
    let source = metadata.interpolate(
        &figment::Profile::Default,
        &["database", "pool_size"],
    );

    println!("database.pool_size came from {source}");
}
```

Para valores suministrados por `ConfiqueEnvProvider`, la interpolación devuelve
el nombre nativo de la variable de entorno declarada en el esquema:

```text
database.pool_size came from APP_DATABASE_POOL_SIZE
```

## Eventos TRACE

El cargador emite eventos de seguimiento de origen con `tracing::trace!`. Lo
hace solo cuando TRACE está habilitado:

```rust
use rust_config_tree::{load_config_with_figment, trace_config_sources};

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;

// If the tracing subscriber is initialized after config loading, emit the
// same source events again after installing the subscriber.
trace_config_sources::<AppConfig>(&figment);
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Cada evento usa el target `rust_config_tree::config` e incluye:

- `config_key`: la clave de configuración con puntos.
- `source`: los metadatos de origen renderizados.

Los valores que provienen solo de valores por defecto de `confique` no tienen
metadatos de tiempo de ejecución de Figment. Se informan como
`confique default or unset optional field`.
