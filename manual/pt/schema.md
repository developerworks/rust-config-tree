# Esquema de configuracao

[English](../en/schema.html) | [中文](../zh/schema.html) | [日本語](../ja/schema.html) | [한국어](../ko/schema.html) | [Français](../fr/schema.html) | [Deutsch](../de/schema.html) | [Español](../es/schema.html) | [Português](schema.html) | [Svenska](../sv/schema.html) | [Suomi](../fi/schema.html) | [Nederlands](../nl/schema.html)

Esquemas de aplicacao sao tipos normais de configuracao `confique`. O esquema
raiz deve implementar `ConfigSchema` para que `rust-config-tree` possa descobrir
includes recursivos a partir da camada intermediaria do `confique`.

```rust
use std::path::PathBuf;

use confique::Config;
use schemars::JsonSchema;
use rust_config_tree::ConfigSchema;

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    database: DatabaseConfig,
}

#[derive(Debug, Config, JsonSchema)]
struct DatabaseConfig {
    #[config(env = "APP_DATABASE_URL")]
    url: String,

    #[config(default = 16)]
    #[config(env = "APP_DATABASE_POOL_SIZE")]
    pool_size: u32,
}

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}
```

## Campo de include

O campo de include pode ter qualquer nome. `rust-config-tree` so o conhece por
meio de `ConfigSchema::include_paths`.

Normalmente, o campo deve ter um padrao vazio:

```rust
#[config(default = [])]
include: Vec<PathBuf>,
```

O carregador recebe uma camada parcialmente carregada para cada arquivo. Isso
permite descobrir arquivos de configuracao filhos antes que o esquema final seja
mesclado e validado.

## Secoes aninhadas

Use `#[config(nested)]` para secoes estruturadas. Secoes aninhadas sempre sao
usadas para carregamento em tempo de execucao. Adicione
`#[schemars(extend("x-tree-split" = true))]` quando um campo aninhado tambem
deve ser gerado como seu proprio modelo `config/*.yaml` e schema
`schemas/*.schema.json`:

```rust
#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    server: ServerConfig,
}
```

O formato YAML natural e:

```yaml
server:
  bind: 127.0.0.1
  port: 8080
```

## Sobrescritas de secao de modelo

Quando uma origem de modelo nao tem includes, o crate pode derivar arquivos de
modelo filhos a partir de secoes de esquema aninhadas marcadas com `x-tree-split`. O caminho padrao de
primeiro nivel e `config/<section>.yaml`.

Sobrescreva esse caminho com `template_path_for_section`:

```rust
impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }

    fn template_path_for_section(section_path: &[&str]) -> Option<PathBuf> {
        match section_path {
            ["database"] => Some(PathBuf::from("examples/database.yaml")),
            _ => None,
        }
    }
}
```

