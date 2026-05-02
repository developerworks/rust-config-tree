# Inicio rápido

[English](../en/quick-start.html) | [中文](../zh/quick-start.html) | [日本語](../ja/quick-start.html) | [한국어](../ko/quick-start.html) | [Français](../fr/quick-start.html) | [Deutsch](../de/quick-start.html) | [Español](quick-start.html) | [Português](../pt/quick-start.html) | [Svenska](../sv/quick-start.html) | [Suomi](../fi/quick-start.html) | [Nederlands](../nl/quick-start.html)

Añade el crate y las bibliotecas de esquema/tiempo de ejecución que usa tu
aplicación:

```toml
[dependencies]
rust-config-tree = "0.1"
confique = { version = "0.4", features = ["yaml", "toml", "json5"] }
figment = { version = "0.10", features = ["yaml", "toml", "json", "env"] }
schemars = { version = "1", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
clap = { version = "4", features = ["derive"] }
```

Define un esquema `confique` e implementa `ConfigSchema` para el tipo raíz:

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

Carga la configuración:

```rust
use rust_config_tree::load_config;

let config = load_config::<AppConfig>("config.yaml")?;
println!("{config:#?}");
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Usa un archivo raíz con includes recursivos:

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

La precedencia por defecto de `load_config` es:

```text
environment variables
  > config files, with later merged files overriding earlier files
    > confique code defaults
```

Cuando los includes se cargan mediante la API de alto nivel, el archivo raíz
tiene la prioridad de archivo más alta. Los archivos incluidos aportan valores
de menor prioridad y pueden usarse para valores por defecto o archivos
específicos de sección.

Los argumentos de línea de comandos son específicos de la aplicación, así que
`load_config` no los lee automáticamente. Fusiona overrides de CLI después de
`build_config_figment` cuando la aplicación tenga flags de override de
configuración:

Los nombres de flags de CLI los elige la aplicación. No son automáticamente
rutas de configuración `a.b.c`. Prefiere flags clap normales como
`--server-port`, luego mapéalas a una estructura de override anidada. La forma
serializada anidada controla la clave de configuración que se sobrescribe.

Solo los valores representados en el proveedor `CliOverrides` de la aplicación
sobrescriben la configuración. Esto es útil para parámetros que se cambian con
frecuencia para una ejecución sin editar el archivo de configuración. Los
valores estables deberían permanecer en archivos de configuración.

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

Con overrides de CLI fusionados de esta forma, la precedencia completa es:

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```
