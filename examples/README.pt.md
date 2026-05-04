# Exemplos

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

Estes exemplos sao pequenos programas executaveis que criam seus proprios
arquivos temporarios de configuracao.

Execute-os a partir da raiz do repositorio:

```bash
cargo run --example basic_loading
cargo run --example cli_overrides -- --server-port 9000
cargo run --example config_commands -- config-template
cargo run --example config_commands -- config-schema
cargo run --example config_commands -- config-validate
cargo run --example generate_templates
cargo run --example tree_api
```

Os exemplos cobrem:

- `basic_loading.rs`: carrega um esquema `confique` a partir de uma arvore de
  configuracao recursiva.
- `cli_overrides.rs`: mescla flags de CLI da aplicacao como o provedor Figment
  de maior prioridade.
- `config_commands.rs`: achata `ConfigCommand` em uma CLI clap da aplicacao.
- `generate_templates.rs`: grava JSON Schemas raiz e de secao, alem de modelos
  TOML/YAML vinculados a esquemas a partir de um esquema.
- `tree_api.rs`: usa a API de arvore de includes de nivel mais baixo e
  independente de formato.

