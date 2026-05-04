# Completions de IDE

[English](../en/ide-completions.html) | [中文](../zh/ide-completions.html) | [日本語](../ja/ide-completions.html) | [한국어](../ko/ide-completions.html) | [Français](../fr/ide-completions.html) | [Deutsch](../de/ide-completions.html) | [Español](../es/ide-completions.html) | [Português](ide-completions.html) | [Svenska](../sv/ide-completions.html) | [Suomi](../fi/ide-completions.html) | [Nederlands](../nl/ide-completions.html)

JSON Schemas gerados podem ser usados por arquivos de configuracao TOML, YAML,
JSON e JSON5. Eles sao gerados a partir do mesmo tipo Rust usado pelo
`confique`:

```rust
use confique::Config;
use schemars::JsonSchema;

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    server: ServerConfig,
}
```

Gere-os com:

```rust
use rust_config_tree::write_config_schemas;

write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Isso grava o esquema raiz e esquemas de secao, como
`schemas/server.schema.json`. Esquemas gerados omitem restricoes `required` para
que o completamento funcione em arquivos de configuracao parciais sem
diagnosticos de campos ausentes. O esquema raiz omite propriedades de secoes
aninhadas, entao o completamento de secoes filhas fica disponivel apenas em
arquivos que vinculam o esquema de secao correspondente.

Campos marcados com `x-env-only` sao omitidos dos esquemas gerados, entao IDEs nao sugerem secrets ou outros valores que devem vir somente de variaveis de ambiente.

Esquemas de IDE servem para completamento e verificacoes basicas do editor,
como tipo, enum e propriedades desconhecidas suportadas pelo esquema gerado.
Eles nao decidem se um valor concreto de campo e valido para a aplicacao. A
validacao de valores deve ser implementada no codigo com
`#[config(validate = Self::validate)]` e executada por `load_config` ou
`config-validate`. Campos obrigatorios e a validacao final da configuracao
mesclada tambem usam esses caminhos de execucao.

## TOML

Arquivos TOML devem vincular o esquema com uma diretiva `#:schema` no topo do
arquivo:

```toml
#:schema ./schemas/myapp.schema.json

[server]
bind = "0.0.0.0"
port = 3000
```

Nao use um campo raiz `$schema = "..."` em TOML. Ele vira dado real de
configuracao e pode afetar a desserializacao em tempo de execucao.
`write_config_templates_with_schema` adiciona a diretiva `#:schema`
automaticamente para modelos TOML.

## YAML

Arquivos YAML devem usar a modeline do YAML Language Server:

```yaml
# yaml-language-server: $schema=./schemas/myapp.schema.json

server:
  bind: 0.0.0.0
  port: 3000
```

`write_config_templates_with_schema` adiciona essa modeline automaticamente para
modelos YAML. Modelos YAML divididos vinculam seu esquema de secao; por exemplo,
`log.yaml` vincula `./schemas/log.schema.json`.

## JSON

Arquivos JSON e JSON5 podem vincular um schema com um campo raiz `$schema`.
`write_config_templates_with_schema` o adiciona automaticamente aos modelos
JSON e JSON5 gerados:

```json
{
  "$schema": "./schemas/myapp.schema.json"
}
```

As configuracoes do editor continuam uteis quando o projeto nao quer vinculo
dentro do arquivo:

```json
{
  "json.schemas": [
    {
      "fileMatch": [
        "/config.json",
        "/config.*.json",
        "/deploy/*.json"
      ],
      "url": "./schemas/myapp.schema.json"
    }
  ]
}
```

YAML tambem pode ser vinculado por configuracoes do VS Code:

```json
{
  "yaml.schemas": {
    "./schemas/myapp.schema.json": [
      "config.yaml",
      "config.*.yaml",
      "deploy/*.yaml"
    ]
  }
}
```

O layout final e:

```text
schemas/myapp.schema.json:
  Root file fields only

schemas/server.schema.json:
  Server section schema

config.toml:
  #:schema ./schemas/myapp.schema.json

config.yaml:
  # yaml-language-server: $schema=./schemas/myapp.schema.json

server.yaml:
  # yaml-language-server: $schema=./schemas/server.schema.json

config.json:
  "$schema": "./schemas/myapp.schema.json"
```

Referencias:

- [Tombi JSON Schema](https://tombi-toml.github.io/tombi/docs/json-schema/)
- [Taplo directives](https://taplo.tamasfe.dev/configuration/directives.html)
- [YAML Language Server](https://github.com/redhat-developer/yaml-language-server)
- [VS Code JSON](https://code.visualstudio.com/Docs/languages/json)
