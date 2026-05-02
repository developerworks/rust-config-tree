# Introducao

[English](../en/introduction.html) | [中文](../zh/introduction.html) | [日本語](../ja/introduction.html) | [한국어](../ko/introduction.html) | [Français](../fr/introduction.html) | [Deutsch](../de/introduction.html) | [Español](../es/introduction.html) | [Português](introduction.html) | [Svenska](../sv/introduction.html) | [Suomi](../fi/introduction.html) | [Nederlands](../nl/introduction.html)

`rust-config-tree` fornece carregamento reutilizavel de arvores de configuracao
e auxiliares de CLI para aplicacoes Rust que usam arquivos de configuracao em
camadas.

O crate e projetado em torno de uma pequena divisao de responsabilidades:

- `confique` e dono das definicoes de esquema, padroes de codigo, validacao e
  geracao de modelos de configuracao.
- `figment` e dono do carregamento em tempo de execucao e dos metadados de
  origem em tempo de execucao.
- `rust-config-tree` e dono da travessia recursiva de includes, resolucao de
  caminhos de include, carregamento de `.env`, descoberta de destinos de modelo
  e comandos clap reutilizaveis.

O crate e util quando uma aplicacao quer um layout natural de arquivos de
configuracao como este:

```yaml
include:
  - config/server.yaml
  - config/database.yaml

log:
  level: info
```

Cada arquivo incluido pode usar o mesmo formato de esquema, e caminhos de
include relativos sao resolvidos a partir do arquivo que os declarou. A
configuracao final continua sendo um valor normal de esquema `confique`.

## Principais recursos

- Travessia recursiva de includes com deteccao de ciclos.
- Caminhos de include relativos resolvidos a partir do arquivo declarante.
- Carregamento de `.env` antes que provedores de ambiente sejam avaliados.
- Variaveis de ambiente declaradas no esquema sem divisao por delimitador.
- Metadados Figment para rastreamento de origem em tempo de execucao.
- Eventos de rastreamento de origem em nivel TRACE por `tracing`.
- Geracao de JSON Schema Draft 7 para completamento e validacao no editor.
- Geracao de modelos YAML, TOML, JSON e JSON5.
- Diretivas TOML `#:schema` e modelines YAML Language Server para modelos
  gerados.
- Divisao automatica de modelos YAML para secoes aninhadas.
- Subcomandos clap embutidos para modelos de configuracao, JSON Schema e shell
  completions.
- Uma API de arvore de nivel mais baixo para chamadores que nao usam
  `confique`.

## Pontos de entrada publicos

Use estas APIs para a maioria das aplicacoes:

- `load_config::<S>(path)` carrega o esquema final.
- `load_config_with_figment::<S>(path)` carrega o esquema e retorna o grafo
  Figment usado para rastreamento de origem.
- `write_config_templates::<S>(config_path, output_path)` grava o modelo raiz e
  modelos filhos descobertos recursivamente.
- `write_config_schemas::<S>(output_path)` grava JSON Schemas Draft 7 raiz e de
  secao.
- `handle_config_command::<Cli, S>(command, config_path)` manipula comandos clap
  de configuracao embutidos.

Use `load_config_tree` quando precisar do primitivo de travessia sem `confique`.

