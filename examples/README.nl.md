# Voorbeelden

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

Deze voorbeelden zijn kleine uitvoerbare programma's die hun eigen tijdelijke
configuratiebestanden maken.

Voer ze uit vanuit de repositoryroot:

```bash
cargo run --example basic_loading
cargo run --example cli_overrides -- --server-port 9000
cargo run --example config_commands -- config-template
cargo run --example config_commands -- config-schema
cargo run --example config_commands -- config-validate
cargo run --example generate_templates
cargo run --example tree_api
```

De voorbeelden behandelen:

- `basic_loading.rs`: laad een `confique`-schema uit een recursieve configuratieboom.
- `cli_overrides.rs`: voeg toepassings-CLI-vlaggen samen als Figment-provider
  met de hoogste prioriteit.
- `config_commands.rs`: flatten `ConfigCommand` in een clap-CLI van een toepassing.
- `generate_templates.rs`: schrijf root- en sectie-JSON Schemas plus
  schemagekoppelde TOML-, YAML-, JSON- en JSON5-sjablonen vanuit een schema.
- `tree_api.rs`: gebruik de lagere, formaatonafhankelijke include-tree-API.
