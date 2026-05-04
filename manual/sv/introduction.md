# Introduktion

[English](../en/introduction.html) | [中文](../zh/introduction.html) | [日本語](../ja/introduction.html) | [한국어](../ko/introduction.html) | [Français](../fr/introduction.html) | [Deutsch](../de/introduction.html) | [Español](../es/introduction.html) | [Português](../pt/introduction.html) | [Svenska](introduction.html) | [Suomi](../fi/introduction.html) | [Nederlands](../nl/introduction.html)

`rust-config-tree` tillhandahaller ateranvandbar laddning av
konfigurationstrad och CLI-hjalpare for Rust-program som anvander
lagerindelade konfigurationsfiler.

Craten bygger pa en liten ansvarsfördelning:

- `confique` ager schemadefinitioner, kodstandardvarden, validering och generering av konfigurationsmallar.
- `figment` ager runtime-laddning och runtime-kallmetadata.
- `rust-config-tree` ager rekursiv include-traversering, include-sokvagsupplosning, `.env`-laddning, upptackt av mallmal och ateranvandbara clap-kommandon.

Craten ar anvandbar nar ett program vill ha en naturlig filstruktur for
konfiguration, till exempel:

```yaml
include:
  - config/server.yaml
  - config/database.yaml

log:
  level: info
```

Varje inkluderad fil kan anvanda samma schemaform, och relativa
include-sokvagar loses fran filen som deklarerade dem. Den slutliga
konfigurationen ar fortfarande ett normalt `confique`-schemavarde.

## Huvudfunktioner

- Rekursiv include-traversering med cykeldetektering.
- Relativa include-sokvagar loses fran deklarerande fil.
- `.env` laddas innan miljo-providers utvarderas.
- Schemadeklarerade miljovariabler utan delimitersplittring.
- Figment-metadata for runtime-kallspArning.
- KallspArningshandelser pa TRACE-niva via `tracing`.
- Draft 7 JSON Schema-generering for editor-komplettering och grundlaggande schemakontroller.
- Faltvardevalidering i programkod med
  `#[config(validate = Self::validate)]`, kord via `load_config` eller
  `config-validate`.
- Generering av YAML-, TOML-, JSON- och JSON5-mallar.
- TOML `#:schema` och YAML Language Server-modelines for genererade mallar.
- Opt-in YAML-malluppdelning for sektioner markerade med `x-tree-split`.
- Inbyggda clap-underkommandon for konfigurationsmallar, JSON Schema och skalkompletteringar.
- Ett lagre niva trad-API for anropare som inte anvander `confique`.

## Publika inganger

Anvand dessa API:er for de flesta program:

- `load_config::<S>(path)` laddar det slutliga schemat.
- `load_config_with_figment::<S>(path)` laddar schemat och returnerar Figment-grafen som anvands for kallspArning.
- `write_config_templates::<S>(config_path, output_path)` skriver rotmallen och rekursivt upptackta barnmallar.
- `write_config_schemas::<S>(output_path)` skriver rot- och sektions-JSON Schemas enligt Draft 7.
- `handle_config_command::<Cli, S>(command, config_path)` hanterar inbyggda clap-konfigurationskommandon.

Anvand `load_config_tree` nar du behover traverseringsprimitiven utan
`confique`.
