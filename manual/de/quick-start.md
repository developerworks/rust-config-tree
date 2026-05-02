# Schnellstart

[English](../en/quick-start.html) | [中文](../zh/quick-start.html) | [日本語](../ja/quick-start.html) | [한국어](../ko/quick-start.html) | [Français](../fr/quick-start.html) | [Deutsch](quick-start.html) | [Español](../es/quick-start.html) | [Português](../pt/quick-start.html) | [Svenska](../sv/quick-start.html) | [Suomi](../fi/quick-start.html) | [Nederlands](../nl/quick-start.html)

Fuege die Crate und die Schema-/Laufzeitbibliotheken hinzu, die deine
Anwendung verwendet:

```toml
[dependencies]
rust-config-tree = "0.1"
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
sie nicht automatisch. Fuehre CLI-Ueberschreibungen nach `build_config_figment`
zusammen, wenn die Anwendung Flags fuer Konfigurationsueberschreibungen hat.

CLI-Flag-Namen werden von der Anwendung gewaehlt. Sie sind nicht automatisch
`a.b.c`-Konfigurationspfade. Bevorzuge normale clap-Flags wie `--server-port`
und bilde sie dann auf eine verschachtelte Ueberschreibungsstruktur ab. Die
verschachtelte serialisierte Form steuert den ueberschriebenen
Konfigurationsschluessel.

Nur Werte, die im `CliOverrides`-Provider der Anwendung dargestellt sind,
ueberschreiben Konfiguration. Das ist nuetzlich fuer Parameter, die haeufig fuer
einen einzelnen Lauf geaendert werden, ohne die Konfigurationsdatei zu
bearbeiten. Stabile Werte sollten in Konfigurationsdateien bleiben.

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

Mit so zusammengefuehrten CLI-Ueberschreibungen ist die vollstaendige
Prioritaet:

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```
