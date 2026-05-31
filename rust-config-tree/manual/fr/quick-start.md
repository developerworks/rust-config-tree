# Demarrage rapide

[English](../en/quick-start.html) | [中文](../zh/quick-start.html) | [日本語](../ja/quick-start.html) | [한국어](../ko/quick-start.html) | [Français](quick-start.html) | [Deutsch](../de/quick-start.html) | [Español](../es/quick-start.html) | [Português](../pt/quick-start.html) | [Svenska](../sv/quick-start.html) | [Suomi](../fi/quick-start.html) | [Nederlands](../nl/quick-start.html)

Ajoutez la crate et les bibliotheques de schema/execution utilisees par votre
application :

```toml
[dependencies]
rust-config-tree = "0.1"
confique = { version = "0.4", features = ["yaml", "toml", "json5"] }
figment = { version = "0.10", features = ["yaml", "toml", "json", "env"] }
schemars = { version = "1", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
clap = { version = "4", features = ["derive"] }
```

Definissez un schema `confique` et implementez `ConfigSchema` pour le type
racine :

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

Chargez la configuration :

```rust
use rust_config_tree::load_config;

let config = load_config::<AppConfig>("config.yaml")?;
println!("{config:#?}");
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Utilisez un fichier racine avec des inclusions recursives :

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

La priorite par defaut de `load_config` est :

```text
environment variables
  > config files, with later merged files overriding earlier files
    > confique code defaults
```

Lorsque les inclusions sont chargees par l'API de haut niveau, le fichier racine
a la priorite de fichier la plus elevee. Les fichiers inclus fournissent des
valeurs de priorite plus faible et peuvent servir de valeurs par defaut ou de
fichiers propres a une section.

Les arguments de ligne de commande sont propres a chaque application, donc
`load_config` ne les lit pas automatiquement. Fusionnez les remplacements CLI
apres `build_config_figment` lorsque l'application possede des drapeaux de
remplacement de configuration :

Les noms de drapeaux CLI sont choisis par l'application. Ils ne sont pas
automatiquement des chemins de configuration `a.b.c`. Preferez des drapeaux
clap normaux comme `--server-port`, puis mappez-les dans une structure de
remplacement imbriquee. La forme serialisee imbriquee controle la cle de
configuration remplacee.

Seules les valeurs representees dans le fournisseur `CliOverrides` de
l'application remplacent la configuration. C'est utile pour les parametres
modifies frequemment pour une seule execution sans modifier le fichier de
configuration. Les valeurs stables doivent rester dans les fichiers de
configuration.

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

Avec des remplacements CLI fusionnes ainsi, la priorite complete est :

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```

