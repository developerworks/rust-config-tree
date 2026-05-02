# rust-config-tree

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

`rust-config-tree` tarjoaa konfiguraatiopuun latauksen ja CLI-apurit Rust-sovelluksille, jotka käyttavat kerrostettuja konfiguraatiotiedostoja.

Projektin opas: <https://developerworks.github.io/rust-config-tree/>.
Kielikohtaiset oppaat julkaistaan itsenaisina mdBook-sivustoina, joissa on kielenvaihtolinkit.

Se kasittelee:

- `confique`-skeeman lataamisen suoraan kaytettavaksi konfiguraatio-olioksi Figmentin runtime provider -lahteiden kautta
- `config-template`-, `completions`- ja `install-completions`-komentojen kasittelijat
- Draft 7 -juuri- ja osio-JSON Schema -skeemojen luonnin editorien taydennysta ja validointia varten
- konfiguraatiomallien luonnin YAML-, TOML-, JSON- ja JSON5-muodoissa
- TOML- ja YAML-mallien skeemadirektiivit ilman runtime-kenttien lisaamista
- rekursiivisen include-lapikaynnin
- `.env`-latauksen ennen ymparistoarvojen yhdistamista
- lahteen seurannan Figment-metadatan kautta
- TRACE-tason lahteen seurannan lokit `tracing`-kirjaston kautta
- suhteelliset include-polut ratkaistuna ne maarittelevasta tiedostosta
- leksikaalisen polun normalisoinnin
- include-syklien tunnistuksen
- deterministisen lapikayntijarjestyksen
- peilatun mallikohteiden keruun
- automaattisen YAML-mallien jakamisen sisakkaisille skeemaosioille

Sovellukset tarjoavat skeemansa johtamalla `confique::Config`-traitin ja toteuttamalla `ConfigSchema`-traitin, joka paljastaa skeeman include-kentan.

## Asennus

```toml
[dependencies]
rust-config-tree = "0.1"
confique = { version = "0.4", features = ["yaml", "toml", "json5"] }
figment = { version = "0.10", features = ["yaml", "env"] }
schemars = { version = "1", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
clap = { version = "4", features = ["derive"] }
```

## Konfiguraatioskeema

Sovelluksen skeema omistaa include-kentan. `rust-config-tree` tarvitsee vain pienen sovittimen, joka poimii includet valiaikaisesta `confique`-kerroksesta.

```rust
use std::path::PathBuf;

use confique::Config;
use rust_config_tree::ConfigSchema;

#[derive(Debug, Config)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(default = "paper")]
    mode: String,

    #[config(nested)]
    server: ServerConfig,
}

#[derive(Debug, Config)]
struct ServerConfig {
    #[config(default = 8080)]
    port: u16,
}

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}
```

Suhteelliset include-polut ratkaistaan tiedostosta, joka ne maarittelee:

```yaml
# config.yaml
include:
  - config/server.yaml

mode: shadow
```

```yaml
# config/server.yaml
server:
  port: 7777
```

Lataa lopullinen skeema `load_config`-funktiolla:

```rust
use rust_config_tree::load_config;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = load_config::<AppConfig>("config.yaml")?;
    println!("{config:#?}");

    Ok(())
}
```

`load_config` lataa ensimmaisen `.env`-tiedoston, joka loytyy kulkemalla ylospain juurikonfiguraatiotiedoston hakemistosta, ennen kuin Figment lukee skeemassa maaritellyt ymparistomuuttujat. Prosessin ymparistossa jo olevat arvot sailyvat ja ovat etusijalla `.env`-arvoihin nahden.

Runtime-lataus tehdaan Figmentin kautta. `confique` vastaa yha skeemametadatasta, oletusarvoista, validoinnista ja mallien luonnista. Ymparistomuuttujien nimet luetaan attribuutista `#[config(env = "...")]`; lataaja ei kayta `Env::split("_")`- tai `Env::split("__")`-jakoa, joten muuttuja kuten `APP_DATABASE_POOL_SIZE` voi vastata kenttaa `database.pool_size`.

`load_config` ei lue komentoriviargumentteja, koska CLI-liput ovat sovelluskohtaisia. Lisaa CLI-ohitukset yhdistamalla provider `build_config_figment`-funktion jalkeen ja validoi sitten `load_config_from_figment`-funktiolla:

CLI-lippujen nimiä ei johdeta konfiguraatiopoluista. Kayta tavallisia sovelluslippuja, kuten `--server-port` tai `--database-url`; ala luota nimiin `--server.port` tai `a.b.c`, ellei sovellus tarkoituksella toteuta sellaista parseria. Sisakkainen serialisoitu ohitusmuoto paattaa, mika konfiguraatioavain ohitetaan.

Vain sovelluksen `CliOverrides`-providerissa esitetyt arvot voivat ohittaa konfiguraation. Tama on tarkoitettu usein saadettaville runtime-parametreille, joissa lipun muuttaminen yhdelle ajolle on parempi kuin konfiguraatiotiedoston muokkaus. Pida pysyva konfiguraatio tiedostoissa ja paljasta CLI-lippuina vain tarkoitukselliset valiaikaiset ohitukset.

```rust
use figment::providers::Serialized;
use serde::Serialize;
use rust_config_tree::{build_config_figment, load_config_from_figment};

#[derive(Debug, Serialize)]
struct CliOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<String>,
}

fn load_with_cli_overrides(cli_mode: Option<String>) -> Result<AppConfig, Box<dyn std::error::Error + Send + Sync>> {
    let cli_overrides = CliOverrides {
        mode: cli_mode,
    };

    let figment = build_config_figment::<AppConfig>("config.yaml")?
        .merge(Serialized::defaults(cli_overrides));

    let config = load_config_from_figment::<AppConfig>(&figment)?;
    Ok(config)
}
```

Kun CLI-ohitukset yhdistetaan talla tavalla, runtime-etusijajarjestys on:

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```

Kayta `load_config_with_figment`-funktiota, kun kutsuja tarvitsee lahdemetadatan:

```rust
use rust_config_tree::load_config_with_figment;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;

    if let Some(metadata) = figment.find_metadata("mode") {
        let source = metadata.interpolate(&figment::Profile::Default, &["mode"]);
        println!("mode came from {source}");
    }

    println!("{config:#?}");

    Ok(())
}
```

Lataaja lahettaa myos konfiguraation lahteenseurannan `tracing::trace!`-tapahtumina. Tapahtumat tuotetaan vain, kun sovelluksen tracing-subscriberissa TRACE on kaytossa. Jos tracing alustetaan konfiguraation latauksen jalkeen, kutsu `trace_config_sources::<AppConfig>(&figment)` subscriberin asentamisen jalkeen.

## Mallien luonti

Mallit renderoidaan samalla skeemalla ja include-lapikaynnin saannoilla. Tulostemuoto paatellaan tulostepolusta:

- `.yaml` ja `.yml` tuottavat YAMLia
- `.toml` tuottaa TOMLia
- `.json` ja `.json5` tuottavat JSON5-yhteensopivia malleja
- tuntematon tai puuttuva paate tuottaa YAMLia

Kayta `write_config_schemas`-funktiota Draft 7 JSON Schema -skeemojen luontiin juurikonfiguraatiolle ja sisakkaisille osioille. Luodut skeemat jattavat `required`-rajoitteet pois, jotta IDEt voivat tarjota taydennysta osittaisille konfiguraatiotiedostoille ilman puuttuvien kenttien virheilmoituksia:

```rust
use rust_config_tree::write_config_schemas;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;

    Ok(())
}
```

Skeemalle, jossa on `server`- ja `log`-osiot, tama kirjoittaa tiedostot `schemas/myapp.schema.json`, `schemas/server.schema.json` ja `schemas/log.schema.json`. Juuriskeema sisaltaa vain juurikonfiguraatiotiedostoon kuuluvat kentat, kuten `include` ja juuritason skalaarikentat. Se jattaa sisakkaisten osioiden ominaisuudet tarkoituksella pois, joten `server` ja `log` taydentyvat vain niiden omia osio-YAML-tiedostoja muokattaessa.

Kayta `write_config_templates`-funktiota juurimallin ja kaikkien sen include-puusta loytyvien mallitiedostojen luontiin:

```rust
use rust_config_tree::write_config_templates;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;

    Ok(())
}
```

Kayta `write_config_templates_with_schema`-funktiota, kun luotujen TOML- ja YAML-mallien tulee sitoa nama skeemat IDE-taydennysta ja validointia varten:

```rust
use rust_config_tree::write_config_templates_with_schema;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_templates_with_schema::<AppConfig>(
        "config.toml",
        "config.example.toml",
        "schemas/myapp.schema.json",
    )?;

    Ok(())
}
```

Juuri-TOML/YAML-kohteet sitovat juuriskeeman eivatka taydenna lapsiosioiden kenttia. Jaetut osio-YAML-kohteet sitovat vastaavan osioskeeman; esimerkiksi `config/log.yaml` saa rivin `# yaml-language-server: $schema=../schemas/log.schema.json`. JSON- ja JSON5-kohteisiin ei tarkoituksella lisata `$schema`-kenttaa; sido ne editoriasetuksilla, kuten VS Coden `json.schemas`.

Mallien luonti valitsee lahdepuun tassa jarjestyksessa:

- olemassa oleva konfiguraatiopolku
- olemassa oleva tulostemallipolku
- tulostepolku, jota kasitellaan uutena tyhjana mallipuuna

Jos lahdesolmulla ei ole include-listaa, `rust-config-tree` johtaa lapsimallitiedostot sisakkaisista `confique`-osioista. Ylla olevalla skeemalla tyhja `config.example.yaml`-lahde tuottaa:

```text
config.example.yaml
config/server.yaml
```

Juurimalli saa include-lohkon tiedostolle `config/server.yaml`. YAML-kohteet, jotka vastaavat sisakkaista osiota, kuten `config/server.yaml`, sisaltavat vain kyseisen osion. Syvemmat sisakkaiset osiot jaetaan rekursiivisesti samalla tavalla.

Ohita `template_path_for_section`, kun osio tulee luoda eri polkuun:

```rust
use std::path::PathBuf;

use confique::Config;
use rust_config_tree::ConfigSchema;

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }

    fn template_path_for_section(section_path: &[&str]) -> Option<PathBuf> {
        match section_path {
            ["server"] => Some(PathBuf::from("examples/server.yaml")),
            _ => None,
        }
    }
}
```

Oletusosiopolku on `config/<section>.yaml` ylatasojen sisakkaisille osioille. Sisakkaiset lapset sijoitetaan vanhemman tiedostonimen rungon alle, esimerkiksi `config/trading/risk.yaml`.

## CLI-integraatio

Litista `ConfigCommand` olemassa olevaan clap-komentoenumiin, jolloin saat:

- `config-template`
- `config-schema`
- `config-validate`
- `completions`
- `install-completions`

Kayttava sovellus sailyttaa oman `Parser`-tyyppinsa ja oman komentoenuminsa. `rust-config-tree` tarjoaa vain uudelleenkaytettavat alikomennot:

1. Lisaa sovelluksen parseriin `#[command(subcommand)] command: Command`.
2. Lisaa sovelluksen `Subcommand`-enumiin `#[command(flatten)] Config(ConfigCommand)`.
3. Clap laajentaa litistetyt variantit samalle alikomentotasolle kuin sovelluksen omat komennot.
4. Tasmaa kyseinen variantti ja kutsu `handle_config_command::<Cli, AppConfig>`.

Sovelluskohtaiset konfiguraation ohitusliput pysyvat sovelluksen omassa parserissa. Esimerkiksi `--server-port` voi vastata avainta `server.port`, kun rakennetaan sisakkainen `CliOverrides { server: Some(CliServerOverrides { port }) }` -arvo ja yhdistetaan se `Serialized::defaults`-providerilla.

```rust
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use confique::Config;
use schemars::JsonSchema;
use rust_config_tree::{ConfigCommand, ConfigSchema, handle_config_command, load_config};

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,
    #[config(default = "paper")]
    mode: String,
}

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}

#[derive(Debug, Parser)]
#[command(name = "demo")]
struct Cli {
    #[arg(long, default_value = "config.yaml")]
    config: PathBuf,
    #[arg(long)]
    server_port: Option<u16>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Run,
    #[command(flatten)]
    Config(ConfigCommand),
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();
    match cli.command {
        Command::Run => {
            let config = load_config::<AppConfig>(&cli.config)?;
            println!("{config:#?}");
        }
        Command::Config(command) => {
            handle_config_command::<Cli, AppConfig>(command, &cli.config)?;
        }
    }

    Ok(())
}
```

`config-template --output <path>` kirjoittaa mallit valittuun polkuun. Jos tulostepolkua ei anneta, se kirjoittaa `config.example.yaml` nykyiseen hakemistoon. Lisaa `--schema <path>`, jotta TOML- ja YAML-mallit sidotaan luotuun JSON Schema -joukkoon ilman runtime-`$schema`-kenttaa. Tama kirjoittaa myos juuriskeeman ja osioskeemat valittuun skeemapolkuun.

`config-schema --output <path>` kirjoittaa Draft 7 -juuri-JSON Schema -skeeman ja osioskeemat. Jos tulostepolkua ei anneta, juuriskeema kirjoitetaan tiedostoon `schemas/config.schema.json`.

`config-validate` lataa koko runtime-konfiguraatiopuun ja ajaa `confique`-oletukset seka validoinnin. Kayta editoriskeemoja hiljaiseen taydennykseen jaettujen tiedostojen muokkauksessa; kayta tata komentoa pakollisille kentille ja lopulliselle konfiguraation validoinnille. Onnistuessaan se tulostaa `Configuration is ok`.

`completions <shell>` tulostaa taydennykset stdoutiin.

`install-completions <shell>` kirjoittaa taydennykset kayttajan kotihakemiston alle ja paivittaa shellin kaynnistystiedoston, kun shell sita vaatii. Bash, Elvish, Fish, PowerShell ja Zsh ovat tuettuja.

## Alemman tason puu-API

Kayta `load_config_tree`-funktiota, kun et kayta `confique`-kirjastoa tai kun tarvitset suoran paasan lapikaynnin tuloksiin:

```rust
use std::{fs, io, path::{Path, PathBuf}};

use rust_config_tree::{ConfigSource, load_config_tree};

fn load_source(path: &Path) -> io::Result<ConfigSource<String>> {
    let content = fs::read_to_string(path)?;
    let includes = content
        .lines()
        .filter_map(|line| line.strip_prefix("include: "))
        .map(PathBuf::from)
        .collect();

    Ok(ConfigSource::new(content, includes))
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let tree = load_config_tree("config.yaml", load_source)?;

    for node in tree.nodes() {
        println!("{}", node.path().display());
    }

    Ok(())
}
```

Puu-API normalisoi polut leksikaalisesti, hylkaa tyhjat include-polut, tunnistaa rekursiiviset include-syklit ja ohittaa tiedostot, jotka on jo ladattu toisen include-haaran kautta.

## Lisenssi

Lisensoitu jommankumman alla:

- Apache License, Version 2.0
- MIT license

valintasi mukaan.
