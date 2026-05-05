# Geracao de modelos

[English](../en/templates.html) | [中文](../zh/templates.html) | [日本語](../ja/templates.html) | [한국어](../ko/templates.html) | [Français](../fr/templates.html) | [Deutsch](../de/templates.html) | [Español](../es/templates.html) | [Português](templates.html) | [Svenska](../sv/templates.html) | [Suomi](../fi/templates.html) | [Nederlands](../nl/templates.html)

Modelos sao gerados a partir do mesmo esquema `confique` usado em tempo de
execucao. `confique` renderiza o conteudo real do modelo, incluindo comentarios
de documentacao, padroes, campos obrigatorios e nomes declarados de variaveis de
ambiente.

Use `write_config_templates`:

```rust
use rust_config_tree::write_config_templates;

write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Gere JSON Schemas Draft 7 para a configuracao raiz e secoes aninhadas marcadas para divisao:

```rust
use rust_config_tree::write_config_schemas;

write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Marque um campo aninhado com `#[schemars(extend("x-tree-split" = true))]`
quando ele deve ser gerado como seu proprio modelo `*.yaml` e seu proprio
schema `<section>.schema.json`. Campos aninhados nao marcados permanecem no
modelo pai e no schema pai.

Marque um campo folha com `#[schemars(extend("x-env-only" = true))]` quando o valor deve vir somente de variaveis de ambiente. Os modelos gerados e os JSON Schemas omitem campos env-only, e objetos pai que ficarem vazios tambem sao removidos.

Os esquemas gerados omitem restricoes `required`. IDEs ainda podem oferecer
completamento, mas arquivos parciais como `log.yaml` nao relatam campos
raiz ausentes. O esquema raiz completa apenas campos que pertencem ao arquivo
raiz; campos de secoes divididas sao omitidos ali e completados por seus
proprios esquemas de secao. Campos presentes ainda podem receber verificacoes
basicas do editor, como tipo, enum e propriedades desconhecidas suportadas pelo
esquema gerado. Os `*.schema.json` gerados nao decidem se um valor concreto de
campo e valido para a aplicacao. A validacao de valores deve ser implementada no
codigo com `#[config(validate = Self::validate)]`; `load_config` e
`config-validate` executam essa validacao em tempo de execucao.

Vincule esses esquemas a partir de modelos TOML, YAML, JSON e JSON5 gerados:

```rust
use rust_config_tree::write_config_templates_with_schema;

write_config_templates_with_schema::<AppConfig>(
    "config.toml",
    "config.example.toml",
    "schemas/myapp.schema.json",
)?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Modelos raiz TOML e YAML vinculam o esquema raiz e nao completam campos de
secoes filhas. Modelos YAML de secao dividida vinculam seu esquema de secao.
Modelos JSON e JSON5 recebem um campo raiz `$schema` que o VS Code pode
reconhecer. VS Code `json.schemas` continua sendo um caminho alternativo de
vinculo.

O formato de saida e inferido a partir do caminho de saida:

- `.yaml` e `.yml` geram YAML.
- `.toml` gera TOML.
- `.json` e `.json5` geram modelos compativeis com JSON5.
- extensoes desconhecidas ou ausentes geram YAML.

## Vinculos de esquema

Com um caminho de esquema `schemas/myapp.schema.json`, modelos raiz gerados usam:

```toml
#:schema ./schemas/myapp.schema.json
```

```yaml
# yaml-language-server: $schema=./schemas/myapp.schema.json
```

Modelos de secao gerados vinculam esquemas de secao:

```yaml
# log.yaml
# yaml-language-server: $schema=./schemas/log.schema.json
```

Modelos JSON e JSON5 gerados escrevem um campo raiz `$schema` reconhecido pelo
VS Code. As configuracoes do editor continuam opcionais:

```json
{
  "json.schemas": [
    {
      "fileMatch": [
        "/config.json",
        "/config.*.json"
      ],
      "url": "./schemas/myapp.schema.json"
    }
  ]
}
```

## Selecao da origem de modelos

A geracao de modelos escolhe sua arvore de origem nesta ordem:

1. Caminho de configuracao existente.
2. Caminho de modelo de saida existente.
3. Caminho de saida tratado como uma nova arvore de modelo vazia.

Isso permite que um projeto atualize modelos a partir dos arquivos de
configuracao atuais, atualize um conjunto de modelos existente ou crie um novo
conjunto de modelos apenas a partir do esquema.

## Arvores de include espelhadas

Se o arquivo de origem declara includes, modelos gerados espelham esses caminhos
de include sob o diretorio de saida.

```yaml
# config.yaml
include:
  - server.yaml
```

Gerar `config.example.yaml` grava:

```text
config.example.yaml
server.yaml
```

Destinos de include relativos sao espelhados sob o diretorio pai do arquivo de
saida. Destinos de include absolutos permanecem absolutos.

## Divisao opt-in de secoes

Quando um arquivo de origem nao tem includes, o crate pode derivar destinos de
include a partir de secoes de esquema aninhadas marcadas com `x-tree-split`. Para um esquema com uma secao marcada
`server`, uma origem de modelo raiz vazia pode produzir:

```text
config.example.yaml
server.yaml
```

O modelo raiz recebe um bloco de include, e `server.yaml` contem apenas a
secao `server`. Secoes aninhadas so sao divididas recursivamente quando esses campos tambem carregam `x-tree-split`.
