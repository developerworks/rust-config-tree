# rust-config-tree

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

`rust-config-tree` biedt het laden van configuratiebomen en CLI-hulpmiddelen
voor Rust-toepassingen die gelaagde configuratiebestanden gebruiken.

Projecthandleiding: <https://developerworks.github.io/rust-config-tree/>.
Taalspecifieke handleidingen worden gepubliceerd als zelfstandige mdBook-sites
met taalschakellinks.

Het ondersteunt:

- het laden van een `confique`-schema naar een direct bruikbaar configuratieobject
  via Figment-runtimeproviders
- opdrachtverwerkers voor `config-template`, `config-schema`,
  `config-validate`, `completions`, `install-completions` en
  `uninstall-completions`
- Draft 7 JSON Schema-generatie voor root- en sectieschema's voor editorcompletion en basale schemacontroles
- configuratiesjabloongeneratie voor YAML, TOML, JSON en JSON5
- schemabindingen voor TOML-, YAML-, JSON- en JSON5-sjablonen
- recursieve include-traversal
- `.env` laden voordat omgevingswaarden worden samengevoegd
- brontracking via Figment-metadata
- brontrackinglogs op TRACE-niveau via `tracing`
- relatieve include-paden opgelost vanuit het bestand dat ze declareert
- lexicale padnormalisatie
- detectie van include-cycli
- deterministische traversalvolgorde
- gespiegeld verzamelen van sjabloondoelen
- opt-in YAML-sjabloonsplitsing voor secties gemarkeerd met `x-tree-split`

Toepassingen leveren hun schema door `confique::Config` af te leiden en
`ConfigSchema` te implementeren om het include-veld van het schema zichtbaar te
maken.

## Installatie

```toml
[dependencies]
rust-config-tree = "0.1"
confique = { version = "0.4", features = ["yaml", "toml", "json5"] }
figment = { version = "0.10", features = ["yaml", "toml", "json", "env"] }
schemars = { version = "1", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
clap = { version = "4", features = ["derive"] }
```

## Configuratieschema

Het toepassingsschema bezit het include-veld. `rust-config-tree` heeft alleen
een kleine adapter nodig die includes uit de tussenliggende `confique`-laag
haalt.

```rust
use std::path::PathBuf;

use confique::Config;
use schemars::JsonSchema;
use rust_config_tree::ConfigSchema;

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(default = "paper")]
    mode: String,

    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    server: ServerConfig,
}

#[derive(Debug, Config, JsonSchema)]
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

Relatieve include-paden worden opgelost vanuit het bestand dat ze declareert:

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

Laad het uiteindelijke schema met `load_config`:

```rust
use rust_config_tree::load_config;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = load_config::<AppConfig>("config.yaml")?;
    println!("{config:#?}");

    Ok(())
}
```

`load_config` laadt het eerste `.env`-bestand dat wordt gevonden door omhoog te
lopen vanaf de directory van het rootconfiguratiebestand, voordat Figment wordt
gevraagd om door het schema gedeclareerde omgevingsvariabelen te lezen. Waarden
die al in de procesomgeving staan, blijven behouden en hebben voorrang op
`.env`-waarden.

Runtimeconfiguratie wordt geladen via Figment. `confique` blijft
verantwoordelijk voor schemametadata, standaardwaarden, validatie en
sjabloongeneratie. Namen van omgevingsvariabelen worden gelezen uit
`#[config(env = "...")]`; de loader gebruikt geen `Env::split("_")` of
`Env::split("__")`, zodat een variabele zoals `APP_DATABASE_POOL_SIZE` kan
verwijzen naar een veld met de naam `database.pool_size`.

`load_config` leest geen commandoregelargumenten omdat CLI-vlaggen
toepassingsspecifiek zijn. Voeg CLI-overrides toe door na
`build_config_figment` een provider samen te voegen en valideer daarna met
`load_config_from_figment`:

CLI-vlagnamen worden niet afgeleid van configuratiepaden. Gebruik normale
toepassingsvlaggen zoals `--server-port` of `--database-url`; vertrouw niet op
`--server.port` of `a.b.c`, tenzij de toepassing die parser bewust
implementeert. De geneste geserialiseerde overridevorm bepaalt welke
configuratiesleutel wordt overschreven.

Alleen waarden die in de `CliOverrides`-provider van de toepassing zijn
opgenomen, kunnen configuratie overschrijven. Dit is bedoeld voor vaak
aangepaste runtimeparameters, waarbij een vlag voor een enkele run beter is dan
een configuratiebestand wijzigen. Houd stabiele configuratie in bestanden en
stel alleen bewuste tijdelijke overrides beschikbaar als CLI-vlaggen.

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

Met CLI-overrides die zo worden samengevoegd, is de runtimeprioriteit:

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```

Gebruik `load_config_with_figment` wanneer de caller bronmetadata nodig heeft:

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

De loader emitteert ook configuratiebrontracking met `tracing::trace!`. Die
events worden alleen geproduceerd wanneer TRACE is ingeschakeld door de
`tracing`-subscriber van de toepassing. Als `tracing` pas na het laden van de
configuratie wordt geinitialiseerd, roep dan
`trace_config_sources::<AppConfig>(&figment)` aan nadat de subscriber is
geinstalleerd.

## Sjabloongeneratie

Sjablonen worden gerenderd met hetzelfde schema en dezelfde include-regels. Het
uitvoerformaat wordt afgeleid uit het uitvoerpad:

- `.yaml` en `.yml` genereren YAML
- `.toml` genereert TOML
- `.json` en `.json5` genereren JSON5-compatibele sjablonen
- onbekende of ontbrekende extensies genereren YAML

Gebruik `write_config_schemas` om Draft 7 JSON Schemas voor de rootconfiguratie
en gesplitste geneste secties te maken. De gegenereerde schema's laten `required`-regels
weg, zodat IDE's completion kunnen bieden voor gedeeltelijke configuratiebestanden
zonder ontbrekende velden te rapporteren. Gegenereerde `*.schema.json`-bestanden
zijn alleen voor IDE-completion en basale editorcontroles; ze bepalen niet of
een concrete veldwaarde geldig is voor de toepassing. Veldwaardevalidatie moet
in code worden geimplementeerd met `#[config(validate = Self::validate)]` en
wordt uitgevoerd via `load_config` of `config-validate`:

```rust
use rust_config_tree::write_config_schemas;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;

    Ok(())
}
```

Markeer een genest veld met `#[schemars(extend("x-tree-split" = true))]`
wanneer het als eigen `*.yaml`-sjabloon en eigen
`<section>.schema.json`-schema moet worden gegenereerd. Ongemarkeerde geneste
velden blijven in het oudersjabloon en het ouderschema.

Markeer een leafveld met `#[schemars(extend("x-env-only" = true))]` wanneer de waarde alleen uit omgevingsvariabelen mag komen. Gegenereerde sjablonen en JSON Schemas laten env-only velden weg, en lege bovenliggende objecten die daardoor overblijven worden verwijderd.

Voor een schema met `server`- en `log`-secties gemarkeerd met `x-tree-split` schrijft dit
`schemas/myapp.schema.json`, `schemas/server.schema.json` en
`schemas/log.schema.json`. Het rootschema bevat alleen velden die in het
rootconfiguratiebestand thuishoren, zoals `include` en scalaire rootvelden. Het
laat gesplitste sectie-eigenschappen bewust weg, zodat `server` en `log` alleen
worden aangevuld wanneer hun eigen sectie-YAML-bestanden worden bewerkt. Ongemarkeerde geneste secties blijven in het rootschema.

Gebruik `write_config_templates` om een rootsjabloon en elk sjabloonbestand dat
via de include-boom bereikbaar is te maken:

```rust
use rust_config_tree::write_config_templates;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;

    Ok(())
}
```

Gebruik `write_config_templates_with_schema` wanneer gegenereerde TOML-, YAML-,
JSON- en JSON5-sjablonen die schema's moeten koppelen voor IDE-completion en
basale schemacontroles:

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

TOML- en YAML-rootdoelen koppelen het rootschema en vullen geen velden van
gesplitste kindsecties aan. Gesplitste sectie-YAML-doelen koppelen hun passende
sectieschema, bijvoorbeeld `log.yaml` krijgt
`# yaml-language-server: $schema=./schemas/log.schema.json`. JSON- en
JSON5-doelen krijgen een rootveld `$schema` dat VS Code kan herkennen.
VS Code `json.schemas` blijft een alternatieve koppelingsroute.

Sjabloongeneratie kiest de bronboom in deze volgorde:

- een bestaand configuratiepad
- een bestaand uitvoersjabloonpad
- het uitvoerpad, behandeld als een nieuwe lege sjabloonboom

Als een bronnode geen include-lijst heeft, leidt `rust-config-tree` kind-
sjabloonbestanden af uit geneste `confique`-secties gemarkeerd met `x-tree-split`. Met het schema hierboven
produceert een lege `config.example.yaml`-bron:

```text
config.example.yaml
server.yaml
```

Het rootsjabloon krijgt een include-blok voor `server.yaml`. YAML-doelen
die naar een geneste sectie verwijzen, zoals `server.yaml`, bevatten
alleen die sectie. Verdere geneste secties worden alleen recursief gesplitst wanneer die velden ook
`x-tree-split` dragen.

Overschrijf `template_path_for_section` wanneer een sectie op een ander pad
moet worden gegenereerd:

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

Het standaardsectiepad is `<section>.yaml` voor geneste secties op het
hoogste niveau. Geneste kinderen worden onder de bestandsstem van hun ouder
geplaatst, bijvoorbeeld `trading/risk.yaml`.

## CLI-integratie

Flatten `ConfigCommand` in de bestaande clap-opdrachtenenum om toe te voegen:

- `config-template`
- `config-schema`
- `config-validate`
- `completions`
- `install-completions`
- `uninstall-completions`

De consumerende toepassing behoudt haar eigen `Parser`-type en eigen
opdrachtenenum. `rust-config-tree` levert alleen herbruikbare subcommands:

1. Voeg `#[command(subcommand)] command: Command` toe aan de parser van de toepassing.
2. Voeg `#[command(flatten)] Config(ConfigCommand)` toe aan de `Subcommand`-enum
   van de toepassing.
3. Clap breidt de geflattende varianten uit naar hetzelfde subcommandniveau als
   de eigen opdrachten van de toepassing.
4. Match die variant en roep `handle_config_command::<Cli, AppConfig>` aan.

Toepassingsspecifieke configuratie-overridevlaggen blijven op de eigen parser
van de toepassing. Bijvoorbeeld: `--server-port` kan naar `server.port` mappen
door een geneste waarde `CliOverrides { server: Some(CliServerOverrides { port }) }`
te bouwen en die met `Serialized::defaults` samen te voegen.

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

`config-template --output <file-name>` schrijft sjablonen onder
`config/<root_config_name>/` met de gekozen bestandsnaam. Als een pad is
opgegeven, wordt alleen de bestandsnaam gebruikt. Als geen uitvoerbestandsnaam
is opgegeven, schrijft het
`config/<root_config_name>/<root_config_name>.example.yaml`. Voeg
`--schema <path>` toe om TOML-, YAML-, JSON- en JSON5-sjablonen te koppelen aan
een gegenereerde JSON Schema-set. JSON- en JSON5-sjablonen krijgen een
`$schema`-veld dat VS Code herkent. Dit schrijft ook het rootschema en de
sectieschema's naar het gekozen schemapad.

`config-schema --output <path>` schrijft het root-Draft 7 JSON Schema en de
sectieschema's. Als geen uitvoerpad is opgegeven, wordt het rootschema naar
`config/<root_config_name>/<root_config_name>.schema.json` geschreven.

`config-validate` laadt de volledige runtimeconfiguratieboom en voert
`confique`-standaardwaarden en validatie uit, inclusief validators die met
`#[config(validate = Self::validate)]` zijn gedeclareerd. Gebruik editorschema's
voor rustige completion tijdens het bewerken van gesplitste bestanden; gebruik
deze opdracht voor vereiste velden en uiteindelijke configuratievalidatie. Het
print `Configuration is ok` wanneer de validatie slaagt.

`completions <shell>` print completions naar stdout.

`install-completions <shell>` schrijft completions onder de home-directory van
de gebruiker en werkt het shellstartbestand bij wanneer de shell dat vereist.
Bash, Elvish, Fish, PowerShell en Zsh worden ondersteund.

`uninstall-completions <shell>` verwijdert het completionbestand van de huidige
binary en verwijdert het beheerde shell-startblok wanneer die shell er een
gebruikt.

## Lagere Tree API

Gebruik `load_config_tree` wanneer je geen `confique` gebruikt of directe
toegang tot traversalresultaten nodig hebt:

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

De tree-API normaliseert paden lexicaal, weigert lege include-paden, detecteert
recursieve include-cycli en slaat bestanden over die al via een andere
include-tak zijn geladen.

## Licentie

Gelicenseerd onder een van:

- Apache License, Version 2.0
- MIT license

naar jouw keuze.
