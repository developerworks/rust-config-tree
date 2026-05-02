# IDE-kompletteringar

[English](../en/ide-completions.html) | [中文](../zh/ide-completions.html) | [日本語](../ja/ide-completions.html) | [한국어](../ko/ide-completions.html) | [Français](../fr/ide-completions.html) | [Deutsch](../de/ide-completions.html) | [Español](../es/ide-completions.html) | [Português](../pt/ide-completions.html) | [Svenska](ide-completions.html) | [Suomi](../fi/ide-completions.html) | [Nederlands](../nl/ide-completions.html)

Genererade JSON Schemas kan anvandas av TOML-, YAML-, JSON- och
JSON5-konfigurationsfiler. De genereras fran samma Rust-typ som anvands av
`confique`:

```rust
use confique::Config;
use schemars::JsonSchema;

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(nested)]
    server: ServerConfig,
}
```

Generera dem med:

```rust
use rust_config_tree::write_config_schemas;

write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Detta skriver rotschemat och sektionsscheman som
`schemas/server.schema.json`. Genererade scheman utelamnar
`required`-begransningar sa komplettering fungerar for partiella
konfigurationsfiler utan diagnostik for saknade falt. Rotschemat utelamnar
nastlade sektionsproperties, sa barnsektionskomplettering ar bara tillganglig i
filer som binder matchande sektionsschema.

IDE-scheman validerar fortfarande befintliga falt, inklusive typ, enum och
kontroller for okanda properties som stods av det genererade schemat. Anvand
`config-validate` for obligatoriska falt och slutlig sammanslagen
konfigurationsvalidering.

## TOML

TOML-filer bor binda schemat med ett `#:schema`-direktiv hogst upp:

```toml
#:schema ./schemas/myapp.schema.json

[server]
bind = "0.0.0.0"
port = 3000
```

Anvand inte ett rot-`$schema = "..."`-falt i TOML. Det blir riktig
konfigurationsdata och kan paverka runtime-deserialisering.
`write_config_templates_with_schema` lagger automatiskt till `#:schema`-
direktivet for TOML-mallar.

## YAML

YAML-filer bor anvanda YAML Language Servers modeline:

```yaml
# yaml-language-server: $schema=./schemas/myapp.schema.json

server:
  bind: 0.0.0.0
  port: 3000
```

`write_config_templates_with_schema` lagger automatiskt till denna modeline for
YAML-mallar. Delade YAML-mallar binder sitt sektionsschema, till exempel binder
`config/log.yaml` `../schemas/log.schema.json`.

## JSON

JSON kan inte bara kommentarer, och `$schema` ar en riktig JSON-property. Hall
runtime-konfigurationsfiler rena och bind JSON-filer via editor-installningar:

```json
{
  "json.schemas": [
    {
      "fileMatch": [
        "/config.json",
        "/config.*.json",
        "/deploy/*.json"
      ],
      "url": "./schemas/myapp.schema.json"
    }
  ]
}
```

YAML kan ocksa bindas via VS Code-installningar:

```json
{
  "yaml.schemas": {
    "./schemas/myapp.schema.json": [
      "config.yaml",
      "config.*.yaml",
      "deploy/*.yaml"
    ]
  }
}
```

Den slutliga layouten ar:

```text
schemas/myapp.schema.json:
  Root file fields only

schemas/server.schema.json:
  Server section schema

config.toml:
  #:schema ./schemas/myapp.schema.json

config.yaml:
  # yaml-language-server: $schema=./schemas/myapp.schema.json

config/server.yaml:
  # yaml-language-server: $schema=../schemas/server.schema.json

config.json:
  No runtime $schema field; bind with editor settings
```

Referenser:

- [Tombi JSON Schema](https://tombi-toml.github.io/tombi/docs/json-schema/)
- [Taplo directives](https://taplo.tamasfe.dev/configuration/directives.html)
- [YAML Language Server](https://github.com/redhat-developer/yaml-language-server)
- [VS Code JSON](https://code.visualstudio.com/Docs/languages/json)
