# IDE-completions

[English](../en/ide-completions.html) | [中文](../zh/ide-completions.html) | [日本語](../ja/ide-completions.html) | [한국어](../ko/ide-completions.html) | [Français](../fr/ide-completions.html) | [Deutsch](../de/ide-completions.html) | [Español](../es/ide-completions.html) | [Português](../pt/ide-completions.html) | [Svenska](../sv/ide-completions.html) | [Suomi](../fi/ide-completions.html) | [Nederlands](ide-completions.html)

Gegenereerde JSON Schemas kunnen worden gebruikt door TOML-, YAML-, JSON- en
JSON5-configuratiebestanden. Ze worden gegenereerd uit hetzelfde Rust-type dat
door `confique` wordt gebruikt:

```rust
use confique::Config;
use schemars::JsonSchema;

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    server: ServerConfig,
}
```

Genereer ze met:

```rust
use rust_config_tree::write_config_schemas;

write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Dit schrijft het rootschema en sectieschema's zoals
`schemas/server.schema.json`. Gegenereerde schema's laten `required`-
constraints weg, zodat completion werkt voor gedeeltelijke configuratiebestanden
zonder diagnostics voor ontbrekende velden. Het rootschema laat geneste
sectie-eigenschappen weg, zodat completion voor kindsecties alleen beschikbaar
is in bestanden die het passende sectieschema koppelen.

Velden gemarkeerd met `x-env-only` worden uit gegenereerde schemas weggelaten, zodat IDEs geen secrets of andere waarden voorstellen die alleen uit omgevingsvariabelen mogen komen.

IDE-schema's zijn voor completion en basale editorcontroles, zoals type-, enum-
en onbekende-eigenschapcontroles die door het gegenereerde schema worden
ondersteund. Ze bepalen niet of een concrete veldwaarde geldig is voor de
toepassing. Veldwaardevalidatie moet in code worden geimplementeerd met
`#[config(validate = Self::validate)]` en wordt uitgevoerd via `load_config` of
`config-validate`. Verplichte velden en uiteindelijke samengevoegde
configuratievalidatie gebruiken ook die runtimepaden.

## TOML

TOML-bestanden moeten het schema koppelen met een `#:schema`-directive bovenaan
het bestand:

```toml
#:schema ./schemas/myapp.schema.json

[server]
bind = "0.0.0.0"
port = 3000
```

Gebruik geen rootveld `$schema = "..."` in TOML. Het wordt echte
configuratiedata en kan runtime-deserialisatie beinvloeden.
`write_config_templates_with_schema` voegt de `#:schema`-directive automatisch
toe voor TOML-sjablonen.

## YAML

YAML-bestanden moeten de YAML Language Server-modeline gebruiken:

```yaml
# yaml-language-server: $schema=./schemas/myapp.schema.json

server:
  bind: 0.0.0.0
  port: 3000
```

`write_config_templates_with_schema` voegt deze modeline automatisch toe voor
YAML-sjablonen. Gesplitste YAML-sjablonen koppelen hun sectieschema, bijvoorbeeld
`log.yaml` koppelt `./schemas/log.schema.json`.

## JSON

JSON- en JSON5-bestanden kunnen een schema koppelen met een rootveld `$schema`. `write_config_templates_with_schema` voegt dit automatisch toe aan gegenereerde JSON- en JSON5-sjablonen:

```json
{
  "$schema": "./schemas/myapp.schema.json"
}
```

Editorinstellingen blijven nuttig wanneer een project geen binding in het bestand wil:

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

YAML kan ook via VS Code-instellingen worden gekoppeld:

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

De uiteindelijke indeling is:

```text
schemas/myapp.schema.json:
  Alleen velden van het rootbestand

schemas/server.schema.json:
  Schema voor de sectie server

config.toml:
  #:schema ./schemas/myapp.schema.json

config.yaml:
  # yaml-language-server: $schema=./schemas/myapp.schema.json

server.yaml:
  # yaml-language-server: $schema=./schemas/server.schema.json

config.json:
  "$schema": "./schemas/myapp.schema.json"
```

Referenties:

- [Tombi JSON Schema](https://tombi-toml.github.io/tombi/docs/json-schema/)
- [Taplo directives](https://taplo.tamasfe.dev/configuration/directives.html)
- [YAML Language Server](https://github.com/redhat-developer/yaml-language-server)
- [VS Code JSON](https://code.visualstudio.com/Docs/languages/json)
