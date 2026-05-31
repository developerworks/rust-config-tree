# ソース追跡

[English](../en/source-tracking.html) | [中文](../zh/source-tracking.html) | [日本語](source-tracking.html) | [한국어](../ko/source-tracking.html) | [Français](../fr/source-tracking.html) | [Deutsch](../de/source-tracking.html) | [Español](../es/source-tracking.html) | [Português](../pt/source-tracking.html) | [Svenska](../sv/source-tracking.html) | [Suomi](../fi/source-tracking.html) | [Nederlands](../nl/source-tracking.html)

runtime loading に使われた Figment graph を保持するには
`load_config_with_figment` を使います。

```rust
use rust_config_tree::load_config_with_figment;

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

返された Figment value は runtime value の source を問い合わせられます。

```rust
if let Some(metadata) = figment.find_metadata("database.pool_size") {
    let source = metadata.interpolate(
        &figment::Profile::Default,
        &["database", "pool_size"],
    );

    println!("database.pool_size came from {source}");
}
```

`ConfiqueEnvProvider` から供給された値の場合、interpolation は schema で宣言
された native environment variable name を返します。

```text
database.pool_size came from APP_DATABASE_POOL_SIZE
```

## TRACE Events

loader は `tracing::trace!` で source tracking event を出力します。TRACE が
有効な場合だけ出力されます。

```rust
use rust_config_tree::{load_config_with_figment, trace_config_sources};

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;

// If the tracing subscriber is initialized after config loading, emit the
// same source events again after installing the subscriber.
trace_config_sources::<AppConfig>(&figment);
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

各 event は `rust_config_tree::config` target を使い、次を含みます。

- `config_key`: dotted config key。
- `source`: rendered source metadata。

`confique` default だけから来た値には Figment runtime metadata がありません。
その場合は `confique default or unset optional field` として報告されます。

