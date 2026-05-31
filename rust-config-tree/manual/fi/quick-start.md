# Pika-aloitus

[English](../en/quick-start.html) | [中文](../zh/quick-start.html) | [日本語](../ja/quick-start.html) | [한국어](../ko/quick-start.html) | [Français](../fr/quick-start.html) | [Deutsch](../de/quick-start.html) | [Español](../es/quick-start.html) | [Português](../pt/quick-start.html) | [Svenska](../sv/quick-start.html) | [Suomi](quick-start.html) | [Nederlands](../nl/quick-start.html)

Lisaa crate ja sovelluksen kayttamat skeema/runtime-kirjastot:

```toml
[dependencies]
rust-config-tree = "0.2"
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

Komentoriviargumentit ovat sovelluskohtaisia, joten `load_config` ei lue niita automaattisesti. Kayta `ConfigOverrides`-johdannais makroa ohitusproviderin rakentamiseen jasetetyista CLI-lipuista:

```rust
use clap::Parser;
use rust_config_tree::{
    ConfigSchema,
    cli::ConfigOverrides,
    config::{build_config_figment, load_config_from_figment},
};

#[derive(Debug, Parser, ConfigOverrides)]
struct Cli {
    #[arg(long)]
    config: Option<std::path::PathBuf>,

    #[arg(long)]
    #[config_override(path = "server.port")]
    server_port: Option<u16>,

    #[arg(long)]
    #[config_override(path = "log.level")]
    log_level: Option<String>,
}

let cli = Cli::parse();
let figment = build_config_figment::<AppConfig>("config.yaml")?
    .merge(cli.config_overrides()?);
let config = load_config_from_figment::<AppConfig>(&figment)?;
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

`#[config_override(path = "...")]`-attribuutti yhdistaa jokaisen CLI-lipun
pisteelliseen konfiguraatiopolkuun. Vain annetut liput tuottavat
ohitusarvoja; pois jaetetyt liput katoavat. Ohitusprovider yhdistetaan
viimeiseksi, joten annetut liput ohittavat tiedosto- ja
ymparistomuuttuja-arvot:

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```
