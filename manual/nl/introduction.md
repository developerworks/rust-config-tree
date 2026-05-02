# Introductie

[English](../en/introduction.html) | [中文](../zh/introduction.html) | [日本語](../ja/introduction.html) | [한국어](../ko/introduction.html) | [Français](../fr/introduction.html) | [Deutsch](../de/introduction.html) | [Español](../es/introduction.html) | [Português](../pt/introduction.html) | [Svenska](../sv/introduction.html) | [Suomi](../fi/introduction.html) | [Nederlands](introduction.html)

`rust-config-tree` biedt herbruikbare configuratieboomlading en CLI-hulpmiddelen
voor Rust-toepassingen die gelaagde configuratiebestanden gebruiken.

De crate is ontworpen rond een kleine verdeling van verantwoordelijkheden:

- `confique` bezit schemadefinities, codestandaarden, validatie en generatie
  van configuratiesjablonen.
- `figment` bezit runtime laden en runtime bronmetadata.
- `rust-config-tree` bezit recursieve include-traversal, oplossen van
  include-paden, `.env` laden, ontdekking van sjabloondoelen en herbruikbare
  clap-opdrachten.

De crate is nuttig wanneer een toepassing een natuurlijke configuratie-indeling
zoals deze wil:

```yaml
include:
  - config/server.yaml
  - config/database.yaml

log:
  level: info
```

Elk geinclude bestand kan dezelfde schemavorm gebruiken, en relatieve
include-paden worden opgelost vanuit het bestand dat ze declareerde. De
uiteindelijke configuratie is nog steeds een normale `confique`-schemawaarde.

## Hoofdfuncties

- Recursieve include-traversal met cycledetectie.
- Relatieve include-paden opgelost vanuit het declarerende bestand.
- `.env` laden voordat omgevingsproviders worden geevalueerd.
- Door het schema gedeclareerde omgevingsvariabelen zonder delimiter-splitting.
- Figment-metadata voor runtime brontracking.
- Brontrackingevents op TRACE-niveau via `tracing`.
- Draft 7 JSON Schema-generatie voor editorcompletion en validatie.
- YAML-, TOML-, JSON- en JSON5-sjabloongeneratie.
- TOML `#:schema` en YAML Language Server-schemamodelines voor gegenereerde sjablonen.
- Automatisch splitsen van YAML-sjablonen voor geneste secties.
- Ingebouwde clap-subcommands voor configuratiesjablonen, JSON Schema en shellcompletions.
- Een lagere tree-API voor callers die geen `confique` gebruiken.

## Publieke entrypoints

Gebruik deze API's voor de meeste toepassingen:

- `load_config::<S>(path)` laadt het uiteindelijke schema.
- `load_config_with_figment::<S>(path)` laadt het schema en retourneert de
  Figment-grafiek die voor brontracking wordt gebruikt.
- `write_config_templates::<S>(config_path, output_path)` schrijft het
  rootsjabloon en recursief ontdekte kindsjablonen.
- `write_config_schemas::<S>(output_path)` schrijft root- en sectie-Draft 7
  JSON Schemas.
- `handle_config_command::<Cli, S>(command, config_path)` verwerkt ingebouwde
  clap-configuratieopdrachten.

Gebruik `load_config_tree` wanneer je de traversalprimitive zonder `confique`
nodig hebt.
