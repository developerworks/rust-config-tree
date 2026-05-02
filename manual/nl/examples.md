# Voorbeelden

[English](../en/examples.html) | [中文](../zh/examples.html) | [日本語](../ja/examples.html) | [한국어](../ko/examples.html) | [Français](../fr/examples.html) | [Deutsch](../de/examples.html) | [Español](../es/examples.html) | [Português](../pt/examples.html) | [Svenska](../sv/examples.html) | [Suomi](../fi/examples.html) | [Nederlands](examples.html)

De repository bevat uitvoerbare voorbeelden voor het laden van configuratiebomen,
CLI-overrides, ingebouwde configuratieopdrachten, sjabloongeneratie en de lagere
tree-API.

Lees de voorbeeldenindex van de repository:

- [examples/README.md](https://github.com/developerworks/rust-config-tree/blob/main/examples/README.md)

Voer voorbeelden uit vanuit de repositoryroot:

```bash
cargo run --example basic_loading
cargo run --example cli_overrides -- --server-port 9000
cargo run --example config_commands -- config-template --output /tmp/config.example.yaml
cargo run --example config_commands -- config-schema --output /tmp/myapp.schema.json
cargo run --example config_commands -- config-validate
cargo run --example generate_templates
cargo run --example tree_api
```
