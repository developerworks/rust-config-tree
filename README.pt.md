# rust-config-tree

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

`rust-config-tree` fornece carregamento de arvores de configuracao e auxiliares
de CLI para aplicacoes Rust que usam arquivos de configuracao em camadas.

Manual do projeto: <https://developerworks.github.io/rust-config-tree/>. Os
manuais de cada idioma sao publicados como sites mdBook independentes com links
para troca de idioma.

Ele lida com:

- carregamento de um esquema `confique` em um objeto de configuracao diretamente
  utilizavel por meio de provedores Figment em tempo de execucao
- manipuladores dos comandos `config-template`, `config-schema`,
  `config-validate`, `completions`, `install-completions` e
  `uninstall-completions`
- geracao de JSON Schema Draft 7 para a raiz e para secoes, para completamento e
  verificacoes basicas de esquema no editor
- geracao de modelos de configuracao para YAML, TOML, JSON e JSON5
- vinculos de esquema para modelos TOML, YAML, JSON e JSON5
- travessia recursiva de includes
- carregamento de `.env` antes que valores de ambiente sejam mesclados
- rastreamento de origem por metadados Figment
- logs de rastreamento de origem em nivel TRACE por `tracing`
- caminhos de include relativos resolvidos a partir do arquivo que os declara
- normalizacao lexical de caminhos
- deteccao de ciclos de include
- ordem de travessia deterministica
- coleta espelhada de destinos de modelo
- divisao opt-in de modelos YAML para secoes marcadas com `x-tree-split`

As aplicacoes fornecem seu esquema derivando `confique::Config` e implementando
`ConfigSchema` para expor o campo de include do esquema.

## Instalacao

```toml
[dependencies]
rust-config-tree = "0.1"
confique = { version = "0.4", features = ["yaml", "toml", "json5"] }
figment = { version = "0.10", features = ["yaml", "toml", "json", "env"] }
schemars = { version = "1", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
clap = { version = "4", features = ["derive"] }
```

## Esquema de configuracao

O esquema da sua aplicacao e dono do campo de include. `rust-config-tree` so
precisa de um pequeno adaptador que extraia includes da camada intermediaria do
`confique`.

```rust
use std::path::PathBuf;

use confique::Config;
use schemars::JsonSchema;
use rust_config_tree::ConfigSchema;

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(default = "paper")]
    mode: String,

    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    server: ServerConfig,
}

#[derive(Debug, Config, JsonSchema)]
struct ServerConfig {
    #[config(default = 8080)]
    port: u16,
}

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}
```

Caminhos de include relativos sao resolvidos a partir do arquivo que os declara:

```yaml
# config.yaml
include:
  - config/server.yaml

mode: shadow
```

```yaml
# config/server.yaml
server:
  port: 7777
```

Carregue o esquema final com `load_config`:

```rust
use rust_config_tree::load_config;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = load_config::<AppConfig>("config.yaml")?;
    println!("{config:#?}");

    Ok(())
}
```

`load_config` carrega o primeiro arquivo `.env` encontrado ao subir a partir do
diretorio do arquivo de configuracao raiz antes de pedir ao Figment que leia as
variaveis de ambiente declaradas no esquema. Valores ja presentes no ambiente do
processo sao preservados e tem precedencia sobre valores de `.env`.

O carregamento de configuracao em tempo de execucao e realizado pelo Figment.
`confique` continua responsavel por metadados de esquema, padroes, validacao e
geracao de modelos. Nomes de variaveis de ambiente sao lidos de
`#[config(env = "...")]`; o carregador nao usa `Env::split("_")` nem
`Env::split("__")`, entao uma variavel como `APP_DATABASE_POOL_SIZE` pode mapear
para um campo chamado `database.pool_size`.

`load_config` nao le argumentos de linha de comando porque flags de CLI sao
especificas da aplicacao. Adicione sobrescritas de CLI mesclando um provedor apos
`build_config_figment` e depois valide com `load_config_from_figment`:

Nomes de flags de CLI nao sao derivados de caminhos de configuracao. Use flags
normais da aplicacao, como `--server-port` ou `--database-url`; nao dependa de
`--server.port` ou `a.b.c` a menos que a aplicacao implemente esse parser
deliberadamente. O formato serializado aninhado da sobrescrita decide qual chave
de configuracao sera sobrescrita.

Somente valores representados no provedor `CliOverrides` da aplicacao podem
sobrescrever a configuracao. Isso e voltado a parametros de tempo de execucao
ajustados com frequencia, quando alterar uma flag em uma execucao e melhor que
editar um arquivo de configuracao. Mantenha configuracao estavel em arquivos e
exponha apenas sobrescritas temporarias deliberadas como flags de CLI.

```rust
use figment::providers::Serialized;
use serde::Serialize;
use rust_config_tree::{build_config_figment, load_config_from_figment};

#[derive(Debug, Serialize)]
struct CliOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<String>,
}

fn load_with_cli_overrides(cli_mode: Option<String>) -> Result<AppConfig, Box<dyn std::error::Error + Send + Sync>> {
    let cli_overrides = CliOverrides {
        mode: cli_mode,
    };

    let figment = build_config_figment::<AppConfig>("config.yaml")?
        .merge(Serialized::defaults(cli_overrides));

    let config = load_config_from_figment::<AppConfig>(&figment)?;
    Ok(config)
}
```

Com sobrescritas de CLI mescladas dessa forma, a precedencia em tempo de
execucao e:

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```

Use `load_config_with_figment` quando o chamador precisar de metadados de
origem:

```rust
use rust_config_tree::load_config_with_figment;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;

    if let Some(metadata) = figment.find_metadata("mode") {
        let source = metadata.interpolate(&figment::Profile::Default, &["mode"]);
        println!("mode came from {source}");
    }

    println!("{config:#?}");

    Ok(())
}
```

O carregador tambem emite rastreamento de origem de configuracao com
`tracing::trace!`. Esses eventos sao produzidos apenas quando TRACE esta
habilitado pelo subscriber de `tracing` da aplicacao. Se `tracing` for
inicializado depois do carregamento da configuracao, chame
`trace_config_sources::<AppConfig>(&figment)` depois de instalar o subscriber.

## Geracao de modelos

Modelos sao renderizados com o mesmo esquema e as mesmas regras de travessia de
includes. O formato de saida e inferido pelo caminho de saida:

- `.yaml` e `.yml` geram YAML
- `.toml` gera TOML
- `.json` e `.json5` geram modelos compativeis com JSON5
- extensoes desconhecidas ou ausentes geram YAML

Use `write_config_schemas` para criar JSON Schemas Draft 7 para a configuracao
raiz e secoes aninhadas marcadas para divisao. Os esquemas gerados omitem restricoes `required` para
que IDEs possam oferecer completamento em arquivos de configuracao parciais sem
relatar campos ausentes. Os arquivos `*.schema.json` gerados servem apenas para
completamento de IDE e verificacoes basicas do editor; eles nao decidem se um
valor concreto de campo e valido para a aplicacao. A validacao de valores deve
ser implementada no codigo com `#[config(validate = Self::validate)]` e
executada por `load_config` ou `config-validate`:

```rust
use rust_config_tree::write_config_schemas;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;

    Ok(())
}
```

Marque um campo aninhado com `#[schemars(extend("x-tree-split" = true))]`
quando ele deve ser gerado como seu proprio modelo `*.yaml` e seu proprio
schema `<section>.schema.json`. Campos aninhados nao marcados permanecem no
modelo pai e no schema pai.

Marque um campo folha com `#[schemars(extend("x-env-only" = true))]` quando o valor deve vir somente de variaveis de ambiente. Os modelos gerados e os JSON Schemas omitem campos env-only, e objetos pai que ficarem vazios tambem sao removidos.

Para um esquema com secoes `server` e `log` marcadas com `x-tree-split`, isso grava
`schemas/myapp.schema.json`, `schemas/server.schema.json` e
`schemas/log.schema.json`. O esquema raiz contem apenas campos que pertencem ao
arquivo de configuracao raiz, como `include` e campos escalares de raiz. Ele
omite intencionalmente propriedades de secoes divididas, entao `server` e `log`
sao completados somente ao editar seus proprios arquivos YAML de secao.

Use `write_config_templates` para criar um modelo raiz e todos os arquivos de
modelo alcancaveis pela arvore de includes:

```rust
use rust_config_tree::write_config_templates;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;

    Ok(())
}
```

Use `write_config_templates_with_schema` quando modelos TOML, YAML, JSON e JSON5
gerados devem vincular esses esquemas para completamento e verificacoes basicas
de esquema no IDE:

```rust
use rust_config_tree::write_config_templates_with_schema;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_templates_with_schema::<AppConfig>(
        "config.toml",
        "config.example.toml",
        "schemas/myapp.schema.json",
    )?;

    Ok(())
}
```

Destinos raiz TOML e YAML vinculam o esquema raiz e nao completam campos de
secoes filhas. Destinos YAML de secao dividida vinculam o esquema da secao
correspondente; por exemplo, `log.yaml` recebe
`# yaml-language-server: $schema=./schemas/log.schema.json`. Destinos JSON e
JSON5 recebem um campo raiz `$schema` que o VS Code pode reconhecer. VS Code
`json.schemas` continua sendo um caminho alternativo de vinculo.

A geracao de modelos escolhe sua arvore de origem nesta ordem:

- um caminho de configuracao existente
- um caminho de modelo de saida existente
- o caminho de saida, tratado como uma nova arvore de modelo vazia

Se um no de origem nao tiver lista de includes, `rust-config-tree` deriva
arquivos de modelo filhos a partir de secoes `confique` aninhadas marcadas com `x-tree-split`. Com o esquema
acima, uma origem `config.example.yaml` vazia produz:

```text
config.example.yaml
server.yaml
```

O modelo raiz recebe um bloco de include para `server.yaml`. Destinos YAML
que mapeiam para uma secao aninhada, como `server.yaml`, contem apenas
essa secao. Secoes ainda mais aninhadas so sao divididas recursivamente quando esses
campos tambem carregam `x-tree-split`.

Sobrescreva `template_path_for_section` quando uma secao deve ser gerada em um
caminho diferente:

```rust
use std::path::PathBuf;

use confique::Config;
use rust_config_tree::ConfigSchema;

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }

    fn template_path_for_section(section_path: &[&str]) -> Option<PathBuf> {
        match section_path {
            ["server"] => Some(PathBuf::from("examples/server.yaml")),
            _ => None,
        }
    }
}
```

O caminho de secao padrao e `<section>.yaml` para secoes aninhadas de
primeiro nivel. Filhos aninhados sao colocados sob o stem do arquivo pai, por
exemplo `trading/risk.yaml`.

## Integracao de CLI

Achate `ConfigCommand` no enum de comandos clap existente para adicionar:

- `config-template`
- `config-schema`
- `config-validate`
- `completions`
- `install-completions`
- `uninstall-completions`

A aplicacao consumidora mantem seu proprio tipo `Parser` e seu proprio enum de
comandos. `rust-config-tree` contribui apenas subcomandos reutilizaveis:

1. Adicione `#[command(subcommand)] command: Command` ao parser da aplicacao.
2. Adicione `#[command(flatten)] Config(ConfigCommand)` ao enum `Subcommand` da
   aplicacao.
3. Clap expande as variantes achatadas para o mesmo nivel de subcomando dos
   comandos proprios da aplicacao.
4. Faca match dessa variante e chame `handle_config_command::<Cli, AppConfig>`.

Flags de sobrescrita de configuracao especificas da aplicacao ficam no proprio
parser da aplicacao. Por exemplo, `--server-port` pode mapear para `server.port`
ao construir um valor aninhado
`CliOverrides { server: Some(CliServerOverrides { port }) }` e mescla-lo com
`Serialized::defaults`.

```rust
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use confique::Config;
use schemars::JsonSchema;
use rust_config_tree::{ConfigCommand, ConfigSchema, handle_config_command, load_config};

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,
    #[config(default = "paper")]
    mode: String,
}

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}

#[derive(Debug, Parser)]
#[command(name = "demo")]
struct Cli {
    #[arg(long, default_value = "config.yaml")]
    config: PathBuf,
    #[arg(long)]
    server_port: Option<u16>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Run,
    #[command(flatten)]
    Config(ConfigCommand),
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();
    match cli.command {
        Command::Run => {
            let config = load_config::<AppConfig>(&cli.config)?;
            println!("{config:#?}");
        }
        Command::Config(command) => {
            handle_config_command::<Cli, AppConfig>(command, &cli.config)?;
        }
    }

    Ok(())
}
```

`config-template --output <file-name>` grava modelos em
`config/<root_config_name>/` usando o nome de arquivo selecionado. Se um caminho
for fornecido, somente o nome do arquivo e usado. Se nenhum nome de arquivo de
saida for fornecido, ele grava
`config/<root_config_name>/<root_config_name>.example.yaml`. Adicione
`--schema <path>` para vincular modelos TOML, YAML, JSON e JSON5 a um conjunto
de JSON Schema gerado. Modelos JSON e JSON5 recebem um campo `$schema`
reconhecido pelo VS Code. Isso tambem grava o esquema raiz e os esquemas de
secao no caminho de esquema selecionado.

`config-schema --output <path>` grava o JSON Schema Draft 7 raiz e esquemas de
secao. Se nenhum caminho de saida for fornecido, o esquema raiz e gravado em
`config/<root_config_name>/<root_config_name>.schema.json`.

`config-validate` carrega a arvore de configuracao completa em tempo de
execucao e executa padroes e validacao do `confique`, incluindo validadores
declarados com `#[config(validate = Self::validate)]`. Use esquemas de editor
para completamento sem ruido ao editar arquivos divididos; use este comando para
campos obrigatorios e validacao final da configuracao. Ele imprime
`Configuration is ok` quando a validacao tem sucesso.

`completions <shell>` imprime completions em stdout.

`install-completions <shell>` grava completions sob o diretorio home do usuario
e atualiza o arquivo de inicializacao do shell quando o shell exige isso. Bash,
Elvish, Fish, PowerShell e Zsh sao suportados.

`uninstall-completions <shell>` remove o arquivo de completion do binario atual
e remove o bloco gerenciado de inicializacao quando esse shell usa um.

## API de arvore de nivel mais baixo

Use `load_config_tree` quando voce nao usa `confique` ou quando precisa de
acesso direto aos resultados da travessia:

```rust
use std::{fs, io, path::{Path, PathBuf}};

use rust_config_tree::{ConfigSource, load_config_tree};

fn load_source(path: &Path) -> io::Result<ConfigSource<String>> {
    let content = fs::read_to_string(path)?;
    let includes = content
        .lines()
        .filter_map(|line| line.strip_prefix("include: "))
        .map(PathBuf::from)
        .collect();

    Ok(ConfigSource::new(content, includes))
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let tree = load_config_tree("config.yaml", load_source)?;

    for node in tree.nodes() {
        println!("{}", node.path().display());
    }

    Ok(())
}
```

A API de arvore normaliza caminhos lexicalmente, rejeita caminhos de include
vazios, detecta ciclos de include recursivos e ignora arquivos que ja foram
carregados por outro ramo de include.

## Licenca

Licenciado sob uma das seguintes, a sua escolha:

- Apache License, Version 2.0
- MIT license
