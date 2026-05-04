# IDE-taydennykset

[English](../en/ide-completions.html) | [中文](../zh/ide-completions.html) | [日本語](../ja/ide-completions.html) | [한국어](../ko/ide-completions.html) | [Français](../fr/ide-completions.html) | [Deutsch](../de/ide-completions.html) | [Español](../es/ide-completions.html) | [Português](../pt/ide-completions.html) | [Svenska](../sv/ide-completions.html) | [Suomi](ide-completions.html) | [Nederlands](../nl/ide-completions.html)

Luotuja JSON Schema -skeemoja voi kayttaa TOML-, YAML-, JSON- ja JSON5-konfiguraatiotiedostoissa. Ne luodaan samasta Rust-tyypista, jota `confique` kayttaa:

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

Luo ne nain:

```rust
use rust_config_tree::write_config_schemas;

write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Tama kirjoittaa juuriskeeman ja osioskeemat, kuten `schemas/server.schema.json`. Luodut skeemat jattavat `required`-rajoitteet pois, jotta taydennys toimii osittaisille konfiguraatiotiedostoille ilman puuttuvien kenttien diagnostiikkaa. Juuriskeema jattaa jaettujen osioiden ominaisuudet pois, joten lapsiosioiden taydennys on saatavilla vain tiedostoissa, jotka sitovat vastaavan osioskeeman. Merkitsemattomat sisakkaiset osiot pysyvat juuriskeemassa.

`x-env-only`-merkityt kentat jatetaan pois luoduista skeemoista, joten IDEt eivat ehdota salaisuuksia tai muita arvoja, joiden tulee tulla vain ymparistomuuttujista.

IDE-skeemat ovat taydennysta ja editorin perustarkistuksia varten, kuten luodun skeeman tukemat tyyppi-, enum- ja tuntemattomien ominaisuuksien tarkistukset. Ne eivat paata, onko konkreettinen kentan arvo sovellukselle kelvollinen. Kentta-arvojen validointi toteutetaan koodissa `#[config(validate = Self::validate)]`-attribuutilla ja suoritetaan `load_config`- tai `config-validate`-polussa. Pakolliset kentat ja lopullisen yhdistetyn konfiguraation validointi kayttavat myos naita runtime-polkuja.

## TOML

TOML-tiedostojen tulisi sitoa skeema tiedoston alun `#:schema`-direktiivilla:

```toml
#:schema ./schemas/myapp.schema.json

[server]
bind = "0.0.0.0"
port = 3000
```

Ala kayta juuritason `$schema = "..."` -kenttaa TOMLissa. Siita tulee oikeaa konfiguraatiodataa ja se voi vaikuttaa runtime-deserialisointiin. `write_config_templates_with_schema` lisaa `#:schema`-direktiivin automaattisesti TOML-malleihin.

## YAML

YAML-tiedostojen tulisi kayttaa YAML Language Server -mallirivia:

```yaml
# yaml-language-server: $schema=./schemas/myapp.schema.json

server:
  bind: 0.0.0.0
  port: 3000
```

`write_config_templates_with_schema` lisaa taman mallirivin automaattisesti YAML-malleihin. Jaetut YAML-mallit sitovat osioskeemansa; esimerkiksi `config/log.yaml` sitoo skeeman `../schemas/log.schema.json`.

## JSON

JSON ei voi sisaltaa kommentteja, ja `$schema` on oikea JSON-ominaisuus. Pida runtime-konfiguraatiotiedostot puhtaina ja sido JSON-tiedostot editoriasetuksilla:

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

YAML voidaan sitoa myos VS Code -asetuksilla:

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

Lopullinen asettelu on:

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

Viitteet:

- [Tombi JSON Schema](https://tombi-toml.github.io/tombi/docs/json-schema/)
- [Taplo directives](https://taplo.tamasfe.dev/configuration/directives.html)
- [YAML Language Server](https://github.com/redhat-developer/yaml-language-server)
- [VS Code JSON](https://code.visualstudio.com/Docs/languages/json)
