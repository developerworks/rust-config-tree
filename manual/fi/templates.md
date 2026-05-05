# Mallien luonti

[English](../en/templates.html) | [中文](../zh/templates.html) | [日本語](../ja/templates.html) | [한국어](../ko/templates.html) | [Français](../fr/templates.html) | [Deutsch](../de/templates.html) | [Español](../es/templates.html) | [Português](../pt/templates.html) | [Svenska](../sv/templates.html) | [Suomi](templates.html) | [Nederlands](../nl/templates.html)

Mallit luodaan samasta `confique`-skeemasta, jota kaytetaan runtimessa. `confique` renderoi varsinaisen mallisisallon, mukaan lukien doc-kommentit, oletusarvot, pakolliset kentat ja maaritellyt ymparistomuuttujien nimet.

Kayta `write_config_templates`-funktiota:

```rust
use rust_config_tree::write_config_templates;

write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Luo Draft 7 JSON Schema -skeemat juurikonfiguraatiolle ja jaetuille sisakkaisille osioille:

```rust
use rust_config_tree::write_config_schemas;

write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Merkitse sisakkainen kentta `#[schemars(extend("x-tree-split" = true))]`-
attribuutilla, kun se tulee luoda omana `*.yaml`-mallina ja omana
`<section>.schema.json`-skeemana. Merkitsemattomat sisakkaiset kentat pysyvat
emomallissa ja emoskeemassa.

Merkitse lehtikentta `#[schemars(extend("x-env-only" = true))]`, kun arvon tulee tulla vain ymparistomuuttujista. Luodut mallit ja JSON Schema -skeemat jattavat env-only-kentat pois, ja niiden takia tyhjiksi jaavat ylaobjektit poistetaan.

Luodut skeemat jattavat `required`-rajoitteet pois. IDEt voivat silti tarjota taydennysta, mutta osittaiset tiedostot kuten `log.yaml` eivat ilmoita puuttuvista juurikentista. Juuriskeema taydentaa vain juuritiedostoon kuuluvat kentat; sisakkaisten osioiden kentat jatetaan siella pois ja taydennetaan niiden omilla osioskeemoilla. Paikalla olevat kentat voivat yha saada editorin perustarkistuksia, kuten luodun skeeman tukemat tyyppi-, enum- ja tuntemattomien ominaisuuksien tarkistukset. Luodut `*.schema.json`-tiedostot eivat paata, onko konkreettinen kentan arvo sovellukselle kelvollinen. Kentta-arvojen validointi toteutetaan koodissa `#[config(validate = Self::validate)]`-attribuutilla; `load_config` ja `config-validate` suorittavat sen runtime-validoinnin.

Sido nama skeemat luoduista TOML-, YAML-, JSON- ja JSON5-malleista:

```rust
use rust_config_tree::write_config_templates_with_schema;

write_config_templates_with_schema::<AppConfig>(
    "config.toml",
    "config.example.toml",
    "schemas/myapp.schema.json",
)?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

TOML- ja YAML-juurimallit sitovat juuriskeeman eivatka taydenna jaettujen
lapsiosioiden kenttia. Jaetut osio-YAML-mallit sitovat oman osioskeemansa.
JSON- ja JSON5-mallit saavat juuritason `$schema`-kentan, jonka VS Code
tunnistaa. VS Coden `json.schemas` on edelleen vaihtoehtoinen sidontatapa.

Tulostemuoto paatellaan tulostepolusta:

- `.yaml` ja `.yml` tuottavat YAMLia.
- `.toml` tuottaa TOMLia.
- `.json` ja `.json5` tuottavat JSON5-yhteensopivia malleja.
- tuntemattomat tai puuttuvat paatteet tuottavat YAMLia.

## Skeemasidonnat

Skeemapolulla `schemas/myapp.schema.json` luodut juurimallit kayttavat:

```toml
#:schema ./schemas/myapp.schema.json
```

```yaml
# yaml-language-server: $schema=./schemas/myapp.schema.json
```

Luodut osiomallit sitovat osioskeemat:

```yaml
# log.yaml
# yaml-language-server: $schema=./schemas/log.schema.json
```

Luodut JSON- ja JSON5-mallit kirjoittavat juuritason `$schema`-kentan, jonka
VS Code tunnistaa. Editoriasetukset ovat edelleen valinnaisia:

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

## Mallilahteen valinta

Mallien luonti valitsee lahdepuunsa tassa jarjestyksessa:

1. Olemassa oleva konfiguraatiopolku.
2. Olemassa oleva tulostemallipolku.
3. Tulostepolku uutena tyhjana mallipuuna.

Tama antaa projektille mahdollisuuden paivittaa malleja nykyisista konfiguraatiotiedostoista, paivittaa olemassa olevan mallijoukon tai luoda uuden mallijoukon pelkasta skeemasta.

## Peilatut include-puut

Jos lahdetiedosto maarittelee includet, luodut mallit peilaavat nama include-polut tulostehakemiston alle.

```yaml
# config.yaml
include:
  - server.yaml
```

`config.example.yaml`-tiedoston luonti kirjoittaa:

```text
config.example.yaml
server.yaml
```

Suhteelliset include-kohteet peilataan tulostetiedoston emohakemiston alle. Absoluuttiset include-kohteet pysyvat absoluuttisina.

## Opt-in-osioiden jakaminen

Kun lahdetiedostolla ei ole includeja, crate voi johtaa include-kohteet `x-tree-split`-merkityista sisakkaisista skeemaosioista. Skeemalle, jossa on merkitty `server`-osio, tyhja juurimallilahde voi tuottaa:

```text
config.example.yaml
server.yaml
```

Juurimalli saa include-lohkon, ja `server.yaml` sisaltaa vain `server`-osion. Sisakkaiset osiot jaetaan rekursiivisesti vain, kun myos niilla kentilla on `x-tree-split`.
