# Runtime-lataus

[English](../en/runtime-loading.html) | [中文](../zh/runtime-loading.html) | [日本語](../ja/runtime-loading.html) | [한국어](../ko/runtime-loading.html) | [Français](../fr/runtime-loading.html) | [Deutsch](../de/runtime-loading.html) | [Español](../es/runtime-loading.html) | [Português](../pt/runtime-loading.html) | [Svenska](../sv/runtime-loading.html) | [Suomi](runtime-loading.html) | [Nederlands](../nl/runtime-loading.html)

Runtime-lataus on tarkoituksella jaettu Figmentin ja confiquen valille:

```text
figment:
  runtime file loading
  runtime environment loading
  runtime source metadata

confique:
  schema metadata
  defaults
  validation
  config templates
```

Paa-API on:

```rust
use rust_config_tree::load_config;

let config = load_config::<AppConfig>("config.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Kayta `load_config_with_figment`-funktiota, kun sovellus tarvitsee lahdemetadatan:

```rust
use rust_config_tree::load_config_with_figment;

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;
# let _ = (config, figment);
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

## Latausvaiheet

Korkean tason lataaja tekee nama vaiheet:

1. Ratkaise juurikonfiguraatiopolku leksikaalisesti.
2. Lataa ensimmainen `.env`-tiedosto, joka loytyy kulkemalla ylospain juurikonfiguraation hakemistosta.
3. Lataa jokainen konfiguraatiotiedosto osittaisena kerroksena includejen loytamiseksi.
4. Rakenna Figment-graafi loydetyista konfiguraatiotiedostoista.
5. Yhdista `ConfiqueEnvProvider` korkeammalla prioriteetilla kuin tiedostot.
6. Yhdista valinnaisesti sovelluskohtaiset CLI-ohitukset.
7. Poimi `confique`-kerros Figmentista.
8. Kayta `confique`-koodin oletusarvoja.
9. Validoi ja rakenna lopullinen skeema.

`load_config` ja `load_config_with_figment` suorittavat vaiheet 1-5 ja 7-9. Vaihe 6 on sovelluskohtainen, koska tama crate ei voi paatella, miten CLI-lippu mapataan skeemakenttaan.

## Tiedostomuodot

Runtime-tiedostoprovider valitaan konfiguraatiopolun paatteesta:

- `.yaml` ja `.yml` kayttavat YAMLia.
- `.toml` kayttaa TOMLia.
- `.json` ja `.json5` kayttavat JSONia.
- tuntemattomat tai puuttuvat paatteet kayttavat YAMLia.

Mallien luonti kayttaa yha confiquen mallirenderoijia YAML-, TOML- ja JSON5-yhteensopivalle tulosteelle.

## Include-prioriteetti

Korkean tason lataaja yhdistaa tiedostoproviderit niin, etta sisallytetyt tiedostot ovat matalammalla prioriteetilla kuin ne sisallyttanyt tiedosto. Juurikonfiguraatiotiedostolla on korkein tiedostoprioriteetti.

Ymparistomuuttujilla on korkeampi prioriteetti kuin kaikilla konfiguraatiotiedostoilla. `confique`-oletuksia kaytetaan vain arvoille, joita runtime-providerit eivat anna.

Kun CLI-ohitukset yhdistetaan `build_config_figment`-funktion jalkeen, koko etusijajarjestys on:

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```

Komentorivisyntaksia ei maarittele `rust-config-tree`. Lippu kuten `--server-port` voi ohittaa `server.port`-avaimen, jos sovellus mapittaa parsitun arvon sisakkaiseen serialisoituun provideriin. Pisteellinen `--server.port`- tai `a.b.c`-syntaksi on olemassa vain, jos sovellus toteuttaa sen.

Tama tarkoittaa, etta CLI-prioriteetti koskee vain avaimia, jotka ovat mukana sovelluksen ohitusproviderissa. Kayta sita operatiivisille arvoille, joita muutetaan usein yhden ajon ajaksi. Jata pysyva konfiguraatio tiedostoihin.

```rust
use figment::providers::Serialized;
use serde::Serialize;
use rust_config_tree::{build_config_figment, load_config_from_figment};

#[derive(Debug, Serialize)]
struct CliOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    server: Option<CliServerOverrides>,
}

#[derive(Debug, Serialize)]
struct CliServerOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    port: Option<u16>,
}

let cli_overrides = CliOverrides {
    server: Some(CliServerOverrides { port: Some(9000) }),
};

let figment = build_config_figment::<AppConfig>("config.yaml")?
    .merge(Serialized::defaults(cli_overrides));

let config = load_config_from_figment::<AppConfig>(&figment)?;
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```
