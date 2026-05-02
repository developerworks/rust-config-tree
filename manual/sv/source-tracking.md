# KallspArning

[English](../en/source-tracking.html) | [中文](../zh/source-tracking.html) | [日本語](../ja/source-tracking.html) | [한국어](../ko/source-tracking.html) | [Français](../fr/source-tracking.html) | [Deutsch](../de/source-tracking.html) | [Español](../es/source-tracking.html) | [Português](../pt/source-tracking.html) | [Svenska](source-tracking.html) | [Suomi](../fi/source-tracking.html) | [Nederlands](../nl/source-tracking.html)

Anvand `load_config_with_figment` for att behalla Figment-grafen som anvands av
runtime-laddningen:

```rust
use rust_config_tree::load_config_with_figment;

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Det returnerade Figment-vardet kan besvara kallfragor for runtime-varden:

```rust
if let Some(metadata) = figment.find_metadata("database.pool_size") {
    let source = metadata.interpolate(
        &figment::Profile::Default,
        &["database", "pool_size"],
    );

    println!("database.pool_size came from {source}");
}
```

For varden som tillhandahalls av `ConfiqueEnvProvider` returnerar interpolation
det ursprungliga miljovariabelnamnet som deklarerats i schemat:

```text
database.pool_size came from APP_DATABASE_POOL_SIZE
```

## TRACE-handelse

Laddaren skickar kallspArningshandelser med `tracing::trace!`. Den gor det bara
nar TRACE ar aktiverat:

```rust
use rust_config_tree::{load_config_with_figment, trace_config_sources};

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;

// If the tracing subscriber is initialized after config loading, emit the
// same source events again after installing the subscriber.
trace_config_sources::<AppConfig>(&figment);
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Varje handelse anvander target `rust_config_tree::config` och innehaller:

- `config_key`: den punktade konfigurationsnyckeln.
- `source`: renderad kallmetadata.

Varden som bara kommer fran `confique`-standardvarden saknar
Figment-runtime-metadata. De rapporteras som
`confique default or unset optional field`.
