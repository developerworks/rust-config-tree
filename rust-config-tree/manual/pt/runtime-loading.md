# Carregamento em tempo de execucao

[English](../en/runtime-loading.html) | [中文](../zh/runtime-loading.html) | [日本語](../ja/runtime-loading.html) | [한국어](../ko/runtime-loading.html) | [Français](../fr/runtime-loading.html) | [Deutsch](../de/runtime-loading.html) | [Español](../es/runtime-loading.html) | [Português](runtime-loading.html) | [Svenska](../sv/runtime-loading.html) | [Suomi](../fi/runtime-loading.html) | [Nederlands](../nl/runtime-loading.html)

O carregamento em tempo de execucao e intencionalmente dividido entre Figment e
confique:

```text
figment:
  runtime file loading
  runtime environment loading
  runtime source metadata

confique:
  schema metadata
  defaults
  validation
  config templates
```

A API principal e:

```rust
use rust_config_tree::load_config;

let config = load_config::<AppConfig>("config.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Use `load_config_with_figment` quando a aplicacao precisar de metadados de
origem:

```rust
use rust_config_tree::load_config_with_figment;

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;
# let _ = (config, figment);
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

## Etapas de carregamento

O carregador de alto nivel executa estas etapas:

1. Resolve lexicalmente o caminho da configuracao raiz.
2. Carrega o primeiro arquivo `.env` encontrado ao subir a partir do diretorio
   da configuracao raiz.
3. Carrega cada arquivo de configuracao como uma camada parcial para descobrir
   includes.
4. Constroi um grafo Figment a partir dos arquivos de configuracao descobertos.
5. Mescla o `ConfiqueEnvProvider` com prioridade maior que arquivos.
6. Opcionalmente mescla sobrescritas de CLI especificas da aplicacao.
7. Extrai uma camada `confique` do Figment.
8. Aplica padroes de codigo do `confique`.
9. Valida e constroi o esquema final.

`load_config` e `load_config_with_figment` executam as etapas 1-5 e 7-9. A
etapa 6 e especifica da aplicacao porque este crate nao consegue inferir como
uma flag de CLI mapeia para um campo do esquema.

## Formatos de arquivo

O provedor de arquivo em tempo de execucao e selecionado pela extensao do
caminho de configuracao:

- `.yaml` e `.yml` usam YAML.
- `.toml` usa TOML.
- `.json` e `.json5` usam JSON.
- extensoes desconhecidas ou ausentes usam YAML.

A geracao de modelos ainda usa os renderizadores de modelo do `confique` para
saida YAML, TOML e compativel com JSON5.

## Prioridade de includes

O carregador de alto nivel mescla provedores de arquivo para que arquivos
incluidos tenham prioridade menor que o arquivo que os incluiu. O arquivo de
configuracao raiz tem a maior prioridade entre arquivos.

Variaveis de ambiente tem prioridade maior que todos os arquivos de
configuracao. Padroes do `confique` sao usados apenas para valores que nao foram
fornecidos por provedores de tempo de execucao.

Quando sobrescritas de CLI sao mescladas depois de `build_config_figment`, a
precedencia completa e:

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```

A sintaxe de linha de comando nao e definida por `rust-config-tree`. Uma flag
como `--server-port` pode sobrescrever `server.port` se a aplicacao mapear esse
valor analisado para um provedor serializado aninhado. Uma sintaxe pontuada
`--server.port` ou `a.b.c` so existe se a aplicacao a implementar.

Isso significa que a precedencia de CLI se aplica apenas a chaves presentes no
provedor de sobrescrita da aplicacao. Use-a para valores operacionais alterados
com frequencia em uma unica execucao. Deixe configuracao duravel em arquivos.

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

