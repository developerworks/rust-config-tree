# Schnellstart

[English](../en/quick-start.html) | [中文](../zh/quick-start.html) | [日本語](../ja/quick-start.html) | [한국어](../ko/quick-start.html) | [Français](../fr/quick-start.html) | [Deutsch](quick-start.html) | [Español](../es/quick-start.html) | [Português](../pt/quick-start.html) | [Svenska](../sv/quick-start.html) | [Suomi](../fi/quick-start.html) | [Nederlands](../nl/quick-start.html)

Fuege die Crate und die Schema-/Laufzeitbibliotheken hinzu, die deine
Anwendung verwendet:

```toml
[dependencies]
rust-config-tree = "0.2"
confique = { version = "0.4", features = ["yaml", "toml", "json5"] }
figment = { version = "0.10", features = ["yaml", "toml", "json", "env"] }
schemars = { version = "1", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
clap = { version = "4", features = ["derive"] }
```

Definiere ein `confique`-Schema und implementiere `ConfigSchema` fuer den
Root-Typ:

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

Lade die Konfiguration:

```rust
use rust_config_tree::load_config;

let config = load_config::<AppConfig>("config.yaml")?;
println!("{config:#?}");
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Verwende eine Root-Datei mit rekursiven Includes:

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

Die Standardprioritaet von `load_config` ist:

```text
environment variables
  > config files, with later merged files overriding earlier files
    > confique code defaults
```

Wenn Includes ueber die High-Level-API geladen werden, hat die Root-Datei die
hoechste Dateiprioritaet. Inkludierte Dateien liefern Werte mit niedrigerer
Prioritaet und koennen fuer Defaults oder abschnittsspezifische Dateien genutzt
werden.

Kommandozeilenargumente sind anwendungsspezifisch, daher liest `load_config`
sie nicht automatisch. Verwende die `ConfigOverrides`-Ableitungsmakro, um
aus geparsten CLI-Flags einen Ueberschreibungsanbieter zu erstellen:

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

Das `#[config_override(path = "...")]`-Attribut bildet jedes CLI-Flag auf
einen gepunkteten Konfigurationspfad ab. Nur angegebene Flags erzeugen
Ueberschreibungswerte; weggelassene Flags verschwinden. Der
Ueberschreibungsanbieter wird zuletzt gemerged, sodass angegebene Flags Datei-
und Umgebungsvariablen ueberschreiben:

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```
