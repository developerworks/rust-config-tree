# Beispiele

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

Diese Beispiele sind kleine ausfuehrbare Programme, die ihre eigenen
temporaeren Konfigurationsdateien erzeugen.

Fuehre sie aus dem Repository-Root aus:

```bash
cargo run --example basic_loading
cargo run --example cli_overrides -- --server-port 9000
cargo run --example config_commands -- config-template --output app_config.example.yaml
cargo run --example config_commands -- config-schema --output /tmp/myapp.schema.json
cargo run --example config_commands -- config-validate
cargo run --example generate_templates
cargo run --example tree_api
```

Die Beispiele decken ab:

- `basic_loading.rs`: laedt ein `confique`-Schema aus einem rekursiven
  Konfigurationsbaum.
- `cli_overrides.rs`: fuehrt Anwendungs-CLI-Flags als Figment-Provider mit
  hoechster Prioritaet zusammen.
- `config_commands.rs`: bindet `ConfigCommand` flach in eine clap-CLI der
  Anwendung ein.
- `generate_templates.rs`: schreibt Root- und Abschnitts-JSON-Schemas sowie
  schemagebundene TOML/YAML-Vorlagen aus einem Schema.
- `tree_api.rs`: verwendet die untergeordnete, formatunabhaengige
  Include-Tree-API.
