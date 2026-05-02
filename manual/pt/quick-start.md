# Inicio rapido

[English](../en/quick-start.html) | [中文](../zh/quick-start.html) | [日本語](../ja/quick-start.html) | [한국어](../ko/quick-start.html) | [Français](../fr/quick-start.html) | [Deutsch](../de/quick-start.html) | [Español](../es/quick-start.html) | [Português](quick-start.html) | [Svenska](../sv/quick-start.html) | [Suomi](../fi/quick-start.html) | [Nederlands](../nl/quick-start.html)

Adicione o crate e as bibliotecas de esquema/tempo de execucao usadas pela sua
aplicacao:

```toml
[dependencies]
rust-config-tree = "0.1"
confique = { version = "0.4", features = ["yaml", "toml", "json5"] }
figment = { version = "0.10", features = ["yaml", "toml", "json", "env"] }
schemars = { version = "1", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
clap = { version = "4", features = ["derive"] }
```

Defina um esquema `confique` e implemente `ConfigSchema` para o tipo raiz:

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

Carregue a configuracao:

```rust
use rust_config_tree::load_config;

let config = load_config::<AppConfig>("config.yaml")?;
println!("{config:#?}");
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Use um arquivo raiz com includes recursivos:

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

A precedencia padrao de `load_config` e:

```text
environment variables
  > config files, with later merged files overriding earlier files
    > confique code defaults
```

Quando includes sao carregados pela API de alto nivel, o arquivo raiz tem a
maior prioridade entre arquivos. Arquivos incluidos fornecem valores de menor
prioridade e podem ser usados para padroes ou arquivos especificos de secao.

Argumentos de linha de comando sao especificos da aplicacao, entao
`load_config` nao os le automaticamente. Mescle sobrescritas de CLI depois de
`build_config_figment` quando a aplicacao tiver flags de sobrescrita de
configuracao:

Nomes de flags de CLI sao escolhidos pela aplicacao. Eles nao sao
automaticamente caminhos de configuracao `a.b.c`. Prefira flags clap normais,
como `--server-port`, e depois mapeie-as para uma estrutura aninhada de
sobrescrita. O formato serializado aninhado controla a chave de configuracao que
sera sobrescrita.

Somente valores representados no provedor `CliOverrides` da aplicacao
sobrescrevem a configuracao. Isso e util para parametros alterados com
frequencia em uma unica execucao sem editar o arquivo de configuracao. Valores
estaveis devem permanecer em arquivos de configuracao.

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

Com sobrescritas de CLI mescladas dessa forma, a precedencia completa e:

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```

