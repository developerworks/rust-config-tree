# Exemplos

[English](../en/examples.html) | [中文](../zh/examples.html) | [日本語](../ja/examples.html) | [한국어](../ko/examples.html) | [Français](../fr/examples.html) | [Deutsch](../de/examples.html) | [Español](../es/examples.html) | [Português](examples.html) | [Svenska](../sv/examples.html) | [Suomi](../fi/examples.html) | [Nederlands](../nl/examples.html)

O repositorio inclui exemplos executaveis para carregar arvores de
configuracao, sobrescritas de CLI, comandos de configuracao embutidos, geracao
de modelos e a API de arvore de nivel mais baixo.

Leia o indice de exemplos do repositorio:

- [examples/README.md](https://github.com/developerworks/rust-config-tree/blob/main/examples/README.md)

Execute exemplos a partir da raiz do repositorio:

```bash
cargo run --example basic_loading
cargo run --example cli_overrides -- --server-port 9000
cargo run --example config_commands -- config-template
cargo run --example config_commands -- config-schema
cargo run --example config_commands -- config-validate
cargo run --example generate_templates
cargo run --example tree_api
```

