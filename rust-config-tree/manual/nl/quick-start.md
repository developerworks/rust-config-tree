# Snelstart

[English](../en/quick-start.html) | [中文](../zh/quick-start.html) | [日本語](../ja/quick-start.html) | [한국어](../ko/quick-start.html) | [Français](../fr/quick-start.html) | [Deutsch](../de/quick-start.html) | [Español](../es/quick-start.html) | [Português](../pt/quick-start.html) | [Svenska](../sv/quick-start.html) | [Suomi](../fi/quick-start.html) | [Nederlands](quick-start.html)

Voeg de crate en de schema/runtime-bibliotheken toe die je toepassing gebruikt:

```toml
[dependencies]
rust-config-tree = "0.2"
confique = { version = "0.4", features = ["yaml", "toml", "json5"] }
figment = { version = "0.10", features = ["yaml", "toml", "json", "env"] }
schemars = { version = "1", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
clap = { version = "4", features = ["derive"] }
```

Definieer een `confique`-schema en implementeer `ConfigSchema` voor het roottype:

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

Laad de configuratie:

```rust
use rust_config_tree::load_config;

let config = load_config::<AppConfig>("config.yaml")?;
println!("{config:#?}");
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Gebruik een rootbestand met recursieve includes:

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

De standaardprioriteit van `load_config` is:

```text
environment variables
  > config files, with later merged files overriding earlier files
    > confique code defaults
```

Wanneer includes door de high-level API worden geladen, heeft het rootbestand
de hoogste bestandsprioriteit. Geinclude bestanden leveren waarden met lagere
prioriteit en kunnen worden gebruikt voor defaults of sectiespecifieke
bestanden.

Commandoregelargumenten zijn toepassingsspecifiek, dus `load_config` leest ze
niet automatisch. Voeg CLI-overrides samen na `build_config_figment` wanneer de
toepassing configuratie-overridevlaggen heeft:

CLI-vlagnamen worden door de toepassing gekozen. Gebruik de `ConfigOverrides`\
derive-macro om een override-provider te bouwen van geparseerde CLI-vlaggen:

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

Het `#[config_override(path = "...")]`-attribuut wijst elke CLI-vlag toe aan een
puntgescheiden configuratiepad. Alleen opgegeven vlaggen produceren
override-waarden; weggelaten vlaggen verdwijnen. De override-provider wordt
als laatste samengevoegd, dus opgegeven vlaggen overschrijven bestands- en
omgevingsvariabelen:

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```
