# Mallgenerering

[English](../en/templates.html) | [中文](../zh/templates.html) | [日本語](../ja/templates.html) | [한국어](../ko/templates.html) | [Français](../fr/templates.html) | [Deutsch](../de/templates.html) | [Español](../es/templates.html) | [Português](../pt/templates.html) | [Svenska](templates.html) | [Suomi](../fi/templates.html) | [Nederlands](../nl/templates.html)

Mallar genereras fran samma `confique`-schema som anvands vid runtime.
`confique` renderar sjalva mallinnehållet, inklusive dokumentationskommentarer,
standardvarden, obligatoriska falt och deklarerade miljovariabelnamn.

Anvand `write_config_templates`:

```rust
use rust_config_tree::write_config_templates;

write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Generera Draft 7 JSON Schemas for rotkonfigurationen och delade nastlade sektioner:

```rust
use rust_config_tree::write_config_schemas;

write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Markera ett nastlat falt med `#[schemars(extend("x-tree-split" = true))]` nar
det ska genereras som en egen `*.yaml`-mall och ett eget
`<section>.schema.json`-schema. Omarkerade nastlade falt stannar i
foraldramallen och foraldraschemat.

Markera ett bladfalt med `#[schemars(extend("x-env-only" = true))]` nar vardet bara ska komma fran miljovariabler. Genererade mallar och JSON Schemas utelamnar env-only-falt, och foralderobjekt som blir tomma tas bort.

Genererade scheman utelamnar `required`-begransningar. IDE:er kan fortfarande
erbjuda komplettering, men partiella filer som `log.yaml` rapporterar
inte saknade rotfalt. Rotschemat kompletterar bara falt som hor hemma i
rotfilen; delade sektionsfalt utelamnas dar och kompletteras av sina egna
sektionsscheman. Befintliga falt kan fortfarande fa grundlaggande
editor-kontroller, som typ, enum och okanda properties som stods av det
genererade schemat. Genererade `*.schema.json`-filer avgor inte om ett konkret
faltvarde ar giltigt for programmet. Faltvardevalidering ska implementeras i kod
med `#[config(validate = Self::validate)]`; `load_config` och `config-validate`
kor den runtime-valideringen.

Bind dessa scheman fran genererade TOML-, YAML-, JSON- och JSON5-mallar:

```rust
use rust_config_tree::write_config_templates_with_schema;

write_config_templates_with_schema::<AppConfig>(
    "config.toml",
    "config.example.toml",
    "schemas/myapp.schema.json",
)?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Rotmallar for TOML och YAML binder rotschemat och kompletterar inte delade
barnsektioners falt. Delade YAML-sektionsmallar binder sina sektionsscheman.
JSON- och JSON5-mallar far ett rotfalt `$schema` som VS Code kan kanna igen.
VS Code `json.schemas` ar fortfarande en alternativ bindningsvag.

Utdataformatet harleds fran utdatasokvagen:

- `.yaml` och `.yml` genererar YAML.
- `.toml` genererar TOML.
- `.json` och `.json5` genererar JSON5-kompatibla mallar.
- okanda eller saknade filandelser genererar YAML.

## Schemabindningar

Med schemasokvagen `schemas/myapp.schema.json` anvander genererade rotmallar:

```toml
#:schema ./schemas/myapp.schema.json
```

```yaml
# yaml-language-server: $schema=./schemas/myapp.schema.json
```

Genererade sektionsmallar binder sektionsscheman:

```yaml
# log.yaml
# yaml-language-server: $schema=./schemas/log.schema.json
```

Genererade JSON- och JSON5-mallar skriver ett rotfalt `$schema` som VS Code
kanner igen. Editor-installningar ar fortfarande valfria:

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

## Val av mallkalla

Mallgenerering valjer sitt kalltrad i denna ordning:

1. Befintlig konfigurationssokvag.
2. Befintlig utdatasokvag for mall.
3. Utdatasokvagen behandlad som ett nytt tomt malltrad.

Det later ett projekt uppdatera mallar fran aktuella konfigurationsfiler,
uppdatera en befintlig malluppsattning eller skapa en ny malluppsattning bara
fran schemat.

## Speglade include-trad

Om kallfilen deklarerar includes speglar genererade mallar dessa
include-sokvagar under utdatakatalogen.

```yaml
# config.yaml
include:
  - server.yaml
```

Generering av `config.example.yaml` skriver:

```text
config.example.yaml
server.yaml
```

Relativa include-mal speglas under utdatafilens foraldrakatalog. Absoluta
include-mal forblir absoluta.

## Opt-in-sektionsuppdelning

Nar en kallfil saknar includes kan craten harleda include-mal fran nastlade
schemasektioner markerade med `x-tree-split`. For ett schema med en markerad `server`-sektion kan en tom rotmallkalla
producera:

```text
config.example.yaml
server.yaml
```

Rotmallen far ett include-block, och `server.yaml` innehaller bara
`server`-sektionen. Nastlade sektioner delas rekursivt bara nar de falten ocksa bar `x-tree-split`.
