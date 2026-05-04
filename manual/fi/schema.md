# Konfiguraatioskeema

[English](../en/schema.html) | [中文](../zh/schema.html) | [日本語](../ja/schema.html) | [한국어](../ko/schema.html) | [Français](../fr/schema.html) | [Deutsch](../de/schema.html) | [Español](../es/schema.html) | [Português](../pt/schema.html) | [Svenska](../sv/schema.html) | [Suomi](schema.html) | [Nederlands](../nl/schema.html)

Sovellusskeemat ovat tavallisia `confique`-konfiguraatiotyyppeja. Juuriskeeman taytyy toteuttaa `ConfigSchema`, jotta `rust-config-tree` voi loytaa rekursiiviset includet valiaikaisesta `confique`-kerroksesta.

```rust
use std::path::PathBuf;

use confique::Config;
use schemars::JsonSchema;
use rust_config_tree::ConfigSchema;

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    database: DatabaseConfig,
}

#[derive(Debug, Config, JsonSchema)]
struct DatabaseConfig {
    #[config(env = "APP_DATABASE_URL")]
    url: String,

    #[config(default = 16)]
    #[config(env = "APP_DATABASE_POOL_SIZE")]
    pool_size: u32,
}

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}
```

## Include-kentta

Include-kentalla voi olla mika tahansa nimi. `rust-config-tree` tuntee sen vain `ConfigSchema::include_paths`-funktion kautta.

Kentalla tulisi tavallisesti olla tyhja oletusarvo:

```rust
#[config(default = [])]
include: Vec<PathBuf>,
```

Lataaja saa jokaisesta tiedostosta osittain ladatun kerroksen. Sen avulla se voi loytaa lapsikonfiguraatiotiedostot ennen lopullisen skeeman yhdistamista ja validointia.

## Sisakkaiset osiot

Kayta `#[config(nested)]`-attribuuttia rakenteisille osioille. Sisakkaisia
osioita kaytetaan aina runtime-lataukseen. Lisaa
`#[schemars(extend("x-tree-split" = true))]`, kun nested-kentta tulee luoda
myos omaksi `*.yaml`-malliksi ja `<section>.schema.json`-skeemaksi:

```rust
#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    server: ServerConfig,
}
```

Luonnollinen YAML-muoto on:

```yaml
server:
  bind: 127.0.0.1
  port: 8080
```

## Vain ymparistomuuttujista tulevat kentat

Merkitse lehtikentta `#[schemars(extend("x-env-only" = true))]`, kun arvon tulee tulla vain ymparistomuuttujasta eika sen pidä nakya luoduissa konfiguraatiotiedostoissa. Luodut YAML-mallit ja JSON Schema -skeemat jattavat env-only-kentat pois, ja niiden takia tyhjiksi jaavat ylaobjektit poistetaan.

```rust
#[config(env = "APP_SECRET")]
#[schemars(extend("x-env-only" = true))]
secret: String,
```

## Kentta-arvojen validointi

Luodut `*.schema.json`-tiedostot ovat vain IDE-taydennysta ja editorin
perustarkistuksia varten. Ne eivat paata, onko konkreettinen kentan arvo
sovellukselle kelvollinen.

Kentta-arvojen validointi toteutetaan koodissa
`#[config(validate = Self::validate)]`-attribuutilla. Validaattori suoritetaan,
kun lopullinen konfiguraatio ladataan `load_config`-funktiolla tai tarkistetaan
`config-validate`-komennolla.

## Malliosioiden ohitukset

Kun mallilahteella ei ole includeja, crate voi johtaa lapsimallitiedostot `x-tree-split`-merkityista sisakkaisista skeemaosioista. Oletuspolku ylatasolla on `<section>.yaml`.

Ohita polku `template_path_for_section`-funktiolla:

```rust
impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }

    fn template_path_for_section(section_path: &[&str]) -> Option<PathBuf> {
        match section_path {
            ["database"] => Some(PathBuf::from("examples/database.yaml")),
            _ => None,
        }
    }
}
```
