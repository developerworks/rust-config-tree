# Ejemplos

[English](../en/examples.html) | [中文](../zh/examples.html) | [日本語](../ja/examples.html) | [한국어](../ko/examples.html) | [Français](../fr/examples.html) | [Deutsch](../de/examples.html) | [Español](examples.html) | [Português](../pt/examples.html) | [Svenska](../sv/examples.html) | [Suomi](../fi/examples.html) | [Nederlands](../nl/examples.html)

El repositorio incluye ejemplos ejecutables para cargar árboles de
configuración, overrides de CLI, comandos de configuración incorporados,
generación de plantillas y la API de árbol de menor nivel.

Lee el índice de ejemplos del repositorio:

- [examples/README.md](https://github.com/developerworks/rust-config-tree/blob/main/examples/README.md)

Ejecuta ejemplos desde la raíz del repositorio:

```bash
cargo run --example basic_loading
cargo run --example cli_overrides -- --server-port 9000
cargo run --example config_commands -- config-template
cargo run --example config_commands -- config-schema
cargo run --example config_commands -- config-validate
cargo run --example generate_templates
cargo run --example tree_api
```
