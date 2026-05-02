# Lahteen seuranta

[English](../en/source-tracking.html) | [中文](../zh/source-tracking.html) | [日本語](../ja/source-tracking.html) | [한국어](../ko/source-tracking.html) | [Français](../fr/source-tracking.html) | [Deutsch](../de/source-tracking.html) | [Español](../es/source-tracking.html) | [Português](../pt/source-tracking.html) | [Svenska](../sv/source-tracking.html) | [Suomi](source-tracking.html) | [Nederlands](../nl/source-tracking.html)

Kayta `load_config_with_figment`-funktiota sailyttaaksesi runtime-latauksessa kaytetyn Figment-graafin:

```rust
use rust_config_tree::load_config_with_figment;

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Palautettu Figment-arvo voi vastata runtime-arvojen lahdekysymyksiin:

```rust
if let Some(metadata) = figment.find_metadata("database.pool_size") {
    let source = metadata.interpolate(
        &figment::Profile::Default,
        &["database", "pool_size"],
    );

    println!("database.pool_size came from {source}");
}
```

`ConfiqueEnvProvider`-providerin antamille arvoille interpolointi palauttaa skeemassa maaritellyn natiivin ymparistomuuttujan nimen:

```text
database.pool_size came from APP_DATABASE_POOL_SIZE
```

## TRACE-tapahtumat

Lataaja lahettaa lahteenseurannan tapahtumia `tracing::trace!`-kutsulla. Se tekee taman vain, kun TRACE on kaytossa:

```rust
use rust_config_tree::{load_config_with_figment, trace_config_sources};

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;

// If the tracing subscriber is initialized after config loading, emit the
// same source events again after installing the subscriber.
trace_config_sources::<AppConfig>(&figment);
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Jokainen tapahtuma kayttaa kohdetta `rust_config_tree::config` ja sisaltaa:

- `config_key`: pisteellinen konfiguraatioavain.
- `source`: renderoitu lahdemetadata.

Arvoilla, jotka tulevat vain `confique`-oletuksista, ei ole Figmentin runtime-metadatan lahdetta. Ne raportoidaan muodossa `confique default or unset optional field`.
