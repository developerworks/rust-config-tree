# Ejemplos

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

Estos ejemplos son pequeños programas ejecutables que crean sus propios
archivos temporales de configuración.

Ejecútalos desde la raíz del repositorio:

```bash
cargo run --example basic_loading
cargo run --example cli_overrides -- --server-port 9000
cargo run --example config_commands -- config-template --output /tmp/config.example.yaml
cargo run --example config_commands -- config-schema --output /tmp/myapp.schema.json
cargo run --example config_commands -- config-validate
cargo run --example generate_templates
cargo run --example tree_api
```

Los ejemplos cubren:

- `basic_loading.rs`: cargar un esquema `confique` desde un árbol de
  configuración recursivo.
- `cli_overrides.rs`: fusionar flags de CLI de la aplicación como proveedor de
  Figment de máxima prioridad.
- `config_commands.rs`: aplanar `ConfigCommand` dentro de una CLI clap de
  aplicación.
- `generate_templates.rs`: escribir JSON Schemas raíz y de sección, además de
  plantillas TOML/YAML vinculadas al esquema desde un esquema.
- `tree_api.rs`: usar la API de árbol de includes de menor nivel e
  independiente del formato.
