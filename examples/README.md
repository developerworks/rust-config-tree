# Examples

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

These examples are small runnable programs that create their own temporary
config files.

Run them from the repository root:

```bash
cargo run --example basic_loading
cargo run --example cli_overrides -- --server-port 9000
cargo run --example config_commands -- config-template
cargo run --example config_commands -- config-schema
cargo run --example config_commands -- config-validate
cargo run --example generate_templates
cargo run --example tree_api
```

The `config_commands` template and schema commands use the CLI defaults, so
`AppConfig` writes generated files under `config/app_config/`.

The examples cover:

- `basic_loading.rs`: load a `confique` schema from a recursive config tree.
- `cli_overrides.rs`: merge application CLI flags as the highest-priority
  Figment provider.
- `config_commands.rs`: flatten `ConfigCommand` into an application clap CLI.
- `generate_templates.rs`: write root and section JSON Schemas plus
  schema-bound TOML/YAML templates from a schema.
- `tree_api.rs`: use the lower-level, format-agnostic include tree API.
