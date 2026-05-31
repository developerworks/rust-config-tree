# Source Tracking

[English](source-tracking.html) | [中文](../zh/source-tracking.html) | [日本語](../ja/source-tracking.html) | [한국어](../ko/source-tracking.html) | [Français](../fr/source-tracking.html) | [Deutsch](../de/source-tracking.html) | [Español](../es/source-tracking.html) | [Português](../pt/source-tracking.html) | [Svenska](../sv/source-tracking.html) | [Suomi](../fi/source-tracking.html) | [Nederlands](../nl/source-tracking.html)

Use `load_config_with_figment` to keep the Figment graph used by runtime
loading:

```rust
use rust_config_tree::load_config_with_figment;

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

The returned Figment value can answer source questions for runtime values:

```rust
if let Some(metadata) = figment.find_metadata("database.pool_size") {
    let source = metadata.interpolate(
        &figment::Profile::Default,
        &["database", "pool_size"],
    );

    println!("database.pool_size came from {source}");
}
```

For values supplied by `ConfiqueEnvProvider`, interpolation returns the native
environment variable name declared in the schema:

```text
database.pool_size came from APP_DATABASE_POOL_SIZE
```

## TRACE Events

The loader emits source tracking events with `tracing::trace!`. It does this
only when TRACE is enabled:

```rust
use rust_config_tree::{load_config_with_figment, trace_config_sources};

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;

// If the tracing subscriber is initialized after config loading, emit the
// same source events again after installing the subscriber.
trace_config_sources::<AppConfig>(&figment);
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Each event uses the `rust_config_tree::config` target and includes:

- `config_key`: the dotted config key.
- `source`: the rendered source metadata.

Values that came only from `confique` defaults do not have Figment runtime
metadata. They are reported as `confique default or unset optional field`.
