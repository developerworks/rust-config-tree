# Exempel

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

Dessa exempel ar sma korbara program som skapar sina egna temporara
konfigurationsfiler.

Kor dem fran repository-roten:

```bash
cargo run --example basic_loading
cargo run --example cli_overrides -- --server-port 9000
cargo run --example config_commands -- config-template --output app_config.example.yaml
cargo run --example config_commands -- config-schema --output /tmp/myapp.schema.json
cargo run --example config_commands -- config-validate
cargo run --example generate_templates
cargo run --example tree_api
```

Exemplen tacker:

- `basic_loading.rs`: ladda ett `confique`-schema fran ett rekursivt konfigurationstrad.
- `cli_overrides.rs`: sla samman programmets CLI-flaggor som Figment-provider med hogsta prioritet.
- `config_commands.rs`: platta ut `ConfigCommand` i ett programs clap-CLI.
- `generate_templates.rs`: skriv rot- och sektions-JSON Schemas samt schemabundna TOML/YAML-mallar fran ett schema.
- `tree_api.rs`: anvand det lagre niva, formatoberoende include-trad-API:t.
