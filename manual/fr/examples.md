# Exemples

[English](../en/examples.html) | [中文](../zh/examples.html) | [日本語](../ja/examples.html) | [한국어](../ko/examples.html) | [Français](examples.html) | [Deutsch](../de/examples.html) | [Español](../es/examples.html) | [Português](../pt/examples.html) | [Svenska](../sv/examples.html) | [Suomi](../fi/examples.html) | [Nederlands](../nl/examples.html)

Le depot inclut des exemples executables pour charger des arbres de
configuration, appliquer des remplacements CLI, utiliser les commandes de
configuration integrees, generer des modeles et utiliser l'API d'arbre de plus
bas niveau.

Lisez l'index des exemples du depot :

- [examples/README.md](https://github.com/developerworks/rust-config-tree/blob/main/examples/README.md)

Executez les exemples depuis la racine du depot :

```bash
cargo run --example basic_loading
cargo run --example cli_overrides -- --server-port 9000
cargo run --example config_commands -- config-template
cargo run --example config_commands -- config-schema
cargo run --example config_commands -- config-validate
cargo run --example generate_templates
cargo run --example tree_api
```

