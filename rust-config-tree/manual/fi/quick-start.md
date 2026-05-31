# Pika-aloitus

[English](../en/quick-start.html) | [中文](../zh/quick-start.html) | [日本語](../ja/quick-start.html) | [한국어](../ko/quick-start.html) | [Français](../fr/quick-start.html) | [Deutsch](../de/quick-start.html) | [Español](../es/quick-start.html) | [Português](../pt/quick-start.html) | [Svenska](../sv/quick-start.html) | [Suomi](quick-start.html) | [Nederlands](../nl/quick-start.html)

Lisaa crate ja sovelluksen kayttamat skeema/runtime-kirjastot:

```toml
[dependencies]
rust-config-tree = "0.1"
confique = { version = "0.4", features = ["yaml", "toml", "json5"] }
figment = { version = "0.10", features = ["yaml", "toml", "json", "env"] }
schemars = { version = "1", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
clap = { version = "4", features = ["derive"] }
```

Maarittele `confique`-skeema ja toteuta `ConfigSchema` juurityypille:

```rust
use std::path::PathBuf;

use confique::Config;
use rust_config_tree::ConfigSchema;

#[derive(Debug, Config)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(nested)]
    server: ServerConfig,
}

#[derive(Debug, Config)]
struct ServerConfig {
    #[config(default = "127.0.0.1")]
    #[config(env = "APP_SERVER_BIND")]
    bind: String,

    #[config(default = 8080)]
    #[config(env = "APP_SERVER_PORT")]
    port: u16,
}

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}
```

Lataa konfiguraatio:

```rust
use rust_config_tree::load_config;

let config = load_config::<AppConfig>("config.yaml")?;
println!("{config:#?}");
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Kayta juuritiedostoa, jossa on rekursiiviset includet:

```yaml
# config.yaml
include:
  - config/server.yaml
```

```yaml
# config/server.yaml
server:
  bind: 0.0.0.0
  port: 3000
```

`load_config`-funktion oletusetusijajarjestys on:

```text
environment variables
  > config files, with later merged files overriding earlier files
    > confique code defaults
```

Kun includet ladataan korkean tason APIlla, juuritiedostolla on korkein tiedostoprioriteetti. Sisallytetyt tiedostot antavat matalamman prioriteetin arvoja ja voivat toimia oletuksina tai osiokohtaisina tiedostoina.

Komentoriviargumentit ovat sovelluskohtaisia, joten `load_config` ei lue niita automaattisesti. Yhdista CLI-ohitukset `build_config_figment`-funktion jalkeen, kun sovelluksella on konfiguraation ohituslippuja:

CLI-lippujen nimet valitsee sovellus. Ne eivat ole automaattisesti `a.b.c`-konfiguraatiopolkuja. Suosi tavallisia clap-lippuja, kuten `--server-port`, ja mapita ne sisakkaiseen ohitusrakenteeseen. Sisakkainen serialisoitu muoto maarittaa ohitettavan konfiguraatioavaimen.

Vain sovelluksen `CliOverrides`-providerissa esitetyt arvot ohittavat konfiguraation. Tama sopii parametreille, joita muutetaan usein yhden ajon ajaksi ilman konfiguraatiotiedoston muokkausta. Pysyvien arvojen tulisi pysya konfiguraatiotiedostoissa.

```rust
use figment::providers::Serialized;
use serde::Serialize;
use rust_config_tree::{build_config_figment, load_config_from_figment};

#[derive(Debug, Serialize)]
struct CliOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    server: Option<CliServerOverrides>,
}

#[derive(Debug, Serialize)]
struct CliServerOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    port: Option<u16>,
}

let cli_overrides = CliOverrides {
    server: Some(CliServerOverrides { port: Some(9000) }),
};

let figment = build_config_figment::<AppConfig>("config.yaml")?
    .merge(Serialized::defaults(cli_overrides));

let config = load_config_from_figment::<AppConfig>(&figment)?;
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Kun CLI-ohitukset yhdistetaan talla tavalla, koko etusijajarjestys on:

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```
