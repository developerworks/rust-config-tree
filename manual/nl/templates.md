# Sjabloongeneratie

[English](../en/templates.html) | [中文](../zh/templates.html) | [日本語](../ja/templates.html) | [한국어](../ko/templates.html) | [Français](../fr/templates.html) | [Deutsch](../de/templates.html) | [Español](../es/templates.html) | [Português](../pt/templates.html) | [Svenska](../sv/templates.html) | [Suomi](../fi/templates.html) | [Nederlands](templates.html)

Sjablonen worden gegenereerd uit hetzelfde `confique`-schema dat runtime wordt
gebruikt. `confique` rendert de daadwerkelijke sjablooninhoud, inclusief
doccomments, defaults, verplichte velden en gedeclareerde omgevingsvariabelen.

Gebruik `write_config_templates`:

```rust
use rust_config_tree::write_config_templates;

write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Genereer Draft 7 JSON Schemas voor de rootconfiguratie en gesplitste geneste secties:

```rust
use rust_config_tree::write_config_schemas;

write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Markeer een genest veld met `#[schemars(extend("x-tree-split" = true))]`
wanneer het als eigen `*.yaml`-sjabloon en eigen
`<section>.schema.json`-schema moet worden gegenereerd. Ongemarkeerde geneste
velden blijven in het oudersjabloon en het ouderschema.

Markeer een leafveld met `#[schemars(extend("x-env-only" = true))]` wanneer de waarde alleen uit omgevingsvariabelen mag komen. Gegenereerde sjablonen en JSON Schemas laten env-only velden weg, en lege bovenliggende objecten die daardoor overblijven worden verwijderd.

Gegenereerde schema's laten `required`-constraints weg. IDE's kunnen nog steeds
completion bieden, maar gedeeltelijke bestanden zoals `log.yaml`
rapporteren geen ontbrekende rootvelden. Het rootschema vult alleen velden aan
die in het rootbestand thuishoren; gesplitste sectievelden worden daar weggelaten
en door hun eigen sectieschema's aangevuld. Aanwezige velden kunnen nog steeds
basale editorcontroles krijgen, zoals type-, enum- en onbekende-eigenschapcontroles
die door het gegenereerde schema worden ondersteund. Gegenereerde
`*.schema.json`-bestanden bepalen niet of een concrete veldwaarde geldig is voor
de toepassing. Veldwaardevalidatie moet in code worden geimplementeerd met
`#[config(validate = Self::validate)]`; `load_config` en `config-validate`
voeren die runtimevalidatie uit.

Koppel die schema's vanuit gegenereerde TOML-, YAML-, JSON- en JSON5-sjablonen:

```rust
use rust_config_tree::write_config_templates_with_schema;

write_config_templates_with_schema::<AppConfig>(
    "config.toml",
    "config.example.toml",
    "schemas/myapp.schema.json",
)?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

TOML- en YAML-rootsjablonen koppelen het rootschema en vullen geen velden van
gesplitste kindsecties aan. Gesplitste sectie-YAML-sjablonen koppelen hun
sectieschema. JSON- en JSON5-sjablonen krijgen een rootveld `$schema` dat
VS Code kan herkennen. VS Code `json.schemas` blijft een alternatieve
koppelingsroute.

Het uitvoerformaat wordt afgeleid uit het uitvoerpad:

- `.yaml` en `.yml` genereren YAML.
- `.toml` genereert TOML.
- `.json` en `.json5` genereren JSON5-compatibele sjablonen.
- onbekende of ontbrekende extensies genereren YAML.

## Schemabindingen

Met een schemapad van `schemas/myapp.schema.json` gebruiken gegenereerde
rootsjablonen:

```toml
#:schema ./schemas/myapp.schema.json
```

```yaml
# yaml-language-server: $schema=./schemas/myapp.schema.json
```

Gegenereerde sectiesjablonen koppelen sectieschema's:

```yaml
# log.yaml
# yaml-language-server: $schema=./schemas/log.schema.json
```

Gegenereerde JSON- en JSON5-sjablonen schrijven een rootveld `$schema` dat
VS Code herkent. Editorinstellingen blijven optioneel:

```json
{
  "json.schemas": [
    {
      "fileMatch": [
        "/config.json",
        "/config.*.json"
      ],
      "url": "./schemas/myapp.schema.json"
    }
  ]
}
```

## Selectie van sjabloonbron

Sjabloongeneratie kiest de bronboom in deze volgorde:

1. Bestaand configuratiepad.
2. Bestaand uitvoersjabloonpad.
3. Uitvoerpad behandeld als een nieuwe lege sjabloonboom.

Daardoor kan een project sjablonen bijwerken vanuit huidige
configuratiebestanden, een bestaande sjabloonset bijwerken of een nieuwe
sjabloonset maken vanuit alleen het schema.

## Gespiegelde include-bomen

Als het bronbestand includes declareert, spiegelen gegenereerde sjablonen die
include-paden onder de uitvoerdirectory.

```yaml
# config.yaml
include:
  - server.yaml
```

Het genereren van `config.example.yaml` schrijft:

```text
config.example.yaml
server.yaml
```

Relatieve include-doelen worden gespiegeld onder de parentdirectory van het
uitvoerbestand. Absolute include-doelen blijven absoluut.

## Opt-in sectiesplitsing

Wanneer een bronbestand geen includes heeft, kan de crate include-doelen
afleiden uit geneste schemaselecties gemarkeerd met `x-tree-split`. Voor een schema met een gemarkeerde `server`-sectie
kan een leeg rootsjabloon bijvoorbeeld produceren:

```text
config.example.yaml
server.yaml
```

Het rootsjabloon krijgt een include-blok en `server.yaml` bevat alleen
de `server`-sectie. Geneste secties worden alleen recursief gesplitst wanneer die velden ook `x-tree-split` dragen.
