# Mallien luonti

[English](../en/templates.html) | [中文](../zh/templates.html) | [日本語](../ja/templates.html) | [한국어](../ko/templates.html) | [Français](../fr/templates.html) | [Deutsch](../de/templates.html) | [Español](../es/templates.html) | [Português](../pt/templates.html) | [Svenska](../sv/templates.html) | [Suomi](templates.html) | [Nederlands](../nl/templates.html)

Mallit luodaan samasta `confique`-skeemasta, jota kaytetaan runtimessa. `confique` renderoi varsinaisen mallisisallon, mukaan lukien doc-kommentit, oletusarvot, pakolliset kentat ja maaritellyt ymparistomuuttujien nimet.

Kayta `write_config_templates`-funktiota:

```rust
use rust_config_tree::write_config_templates;

write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Luo Draft 7 JSON Schema -skeemat juurikonfiguraatiolle ja sisakkaisille osioille:

```rust
use rust_config_tree::write_config_schemas;

write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Luodut skeemat jattavat `required`-rajoitteet pois. IDEt voivat silti tarjota taydennysta, mutta osittaiset tiedostot kuten `config/log.yaml` eivat ilmoita puuttuvista juurikentista. Juuriskeema taydentaa vain juuritiedostoon kuuluvat kentat; sisakkaisten osioiden kentat jatetaan siella pois ja taydennetaan niiden omilla osioskeemoilla. Paikalla olevat kentat tarkistetaan yha skeemalla IDEssa. Pakolliset kentat ja lopullinen yhdistetyn konfiguraation validointi hoidetaan `load_config`-funktiolla tai `config-validate`-komennolla.

Sido nama skeemat luoduista TOML- ja YAML-malleista:

```rust
use rust_config_tree::write_config_templates_with_schema;

write_config_templates_with_schema::<AppConfig>(
    "config.toml",
    "config.example.toml",
    "schemas/myapp.schema.json",
)?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Juuri-TOML/YAML-mallit sitovat juuriskeeman eivatka taydenna lapsiosioiden kenttia. Jaetut osio-YAML-mallit sitovat oman osioskeemansa. JSON- ja JSON5-mallit jatetaan muuttamatta, jotta runtime-konfiguraatio ei sisalla `$schema`-kenttaa. Sido JSON-tiedostot editoriasetuksilla, kuten VS Coden `json.schemas`.

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
# config/log.yaml
# yaml-language-server: $schema=../schemas/log.schema.json
```

JSONissa tiedosto pidetaan vapaana `$schema`-kentasta ja sidotaan editoriasetuksilla:

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
  - config/server.yaml
```

`config.example.yaml`-tiedoston luonti kirjoittaa:

```text
config.example.yaml
config/server.yaml
```

Suhteelliset include-kohteet peilataan tulostetiedoston emohakemiston alle. Absoluuttiset include-kohteet pysyvat absoluuttisina.

## Automaattinen osioiden jakaminen

Kun lahdetiedostolla ei ole includeja, crate voi johtaa include-kohteet sisakkaisista skeemaosioista. Skeemalle, jossa on `server`-osio, tyhja juurimallilahde voi tuottaa:

```text
config.example.yaml
config/server.yaml
```

Juurimalli saa include-lohkon, ja `config/server.yaml` sisaltaa vain `server`-osion. Sisakkaiset osiot jaetaan rekursiivisesti.
