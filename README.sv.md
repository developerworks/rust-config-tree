# rust-config-tree

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

`rust-config-tree` tillhandahaller laddning av konfigurationstrad och
CLI-hjalpare for Rust-program som anvander lagerindelade konfigurationsfiler.

Projektmanual: <https://developerworks.github.io/rust-config-tree/>. Manualer
publiceras som fristaende mdBook-webbplatser med spraklankar.

Det hanterar:

- laddning av ett `confique`-schema till ett direkt anvandbart konfigurationsobjekt via Figment-runtime providers
- kommandohanterare for `config-template`, `config-schema`,
  `config-validate`, `completions`, `install-completions` och
  `uninstall-completions`
- generering av Draft 7 JSON Schema for rot och sektioner for editor-komplettering och grundlaggande schemakontroller
- generering av konfigurationsmallar for YAML, TOML, JSON och JSON5
- schemabindningar for TOML-, YAML-, JSON- och JSON5-mallar
- rekursiv include-traversering
- laddning av `.env` innan miljo-varden slas samman
- kallspArning via Figment-metadata
- kallspArningsloggar pa TRACE-niva via `tracing`
- relativa include-sokvagar upplosta fran filen som deklarerar dem
- lexikal sokvagsnormalisering
- detektering av include-cykler
- deterministisk traverseringsordning
- speglad insamling av mallmal
- opt-in YAML-malluppdelning for sektioner markerade med `x-tree-split`

Program tillhandahaller sitt schema genom att derivera `confique::Config` och
implementera `ConfigSchema` for att exponera schemats include-falt.

## Installation

```toml
[dependencies]
rust-config-tree = "0.1"
confique = { version = "0.4", features = ["yaml", "toml", "json5"] }
figment = { version = "0.10", features = ["yaml", "toml", "json", "env"] }
schemars = { version = "1", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
clap = { version = "4", features = ["derive"] }
```

## Konfigurationsschema

Ditt programschema ager include-faltet. `rust-config-tree` behover bara en
liten adapter som hamtar includes fran det mellanliggande `confique`-lagret.

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

Relativa include-sokvagar losas fran filen som deklarerar dem:

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

Ladda det slutliga schemat med `load_config`:

```rust
use rust_config_tree::load_config;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = load_config::<AppConfig>("config.yaml")?;
    println!("{config:#?}");

    Ok(())
}
```

`load_config` laddar den forsta `.env`-filen som hittas genom att ga uppat fran
rotkonfigurationsfilens katalog innan Figment far lasa schemadeklarerade
miljovariabler. Varden som redan finns i processmiljon bevaras och har hogre
prioritet an `.env`-varden.

Runtime-laddning av konfiguration sker via Figment. `confique` ansvarar fortsatt
for schemametadata, standardvarden, validering och mallgenerering.
Miljovariabelnamn lases fran `#[config(env = "...")]`; laddaren anvander inte
`Env::split("_")` eller `Env::split("__")`, sa en variabel som
`APP_DATABASE_POOL_SIZE` kan mappa till ett falt med namnet
`database.pool_size`.

`load_config` laser inte kommandoradsargument eftersom CLI-flaggor ar
programspecifika. Lagg till CLI-override genom att sla samman en provider efter
`build_config_figment`, och validera sedan med `load_config_from_figment`:

CLI-flaggornas namn harleds inte fran konfigurationssokvagar. Anvand normala
programflaggor som `--server-port` eller `--database-url`; forlita dig inte pa
`--server.port` eller `a.b.c` om inte programmet avsiktligt implementerar en
sadan parser. Den nastlade serialiserade override-formen avgor vilken
konfigurationsnyckel som skrivs over.

Endast varden som representeras i programmets `CliOverrides`-provider kan
skriva over konfiguration. Detta ar avsett for runtime-parametrar som justeras
ofta, dar en flagga for en enskild korning ar battre an att redigera en
konfigurationsfil. Hall stabil konfiguration i filer och exponera bara
avsiktliga tillfalliga overrides som CLI-flaggor.

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

Med CLI-overrides sammanslagna pa detta satt ar runtime-prioriteten:

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```

Anvand `load_config_with_figment` nar anroparen behover kallmetadata:

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

Laddaren skickar ocksa kallspArning for konfiguration med `tracing::trace!`.
Dessa handelser produceras bara nar TRACE ar aktiverat i programmets
tracing-subscriber. Om tracing initieras efter konfigurationsladdning, anropa
`trace_config_sources::<AppConfig>(&figment)` efter att subscribern installerats.

## Mallgenerering

Mallar renderas med samma schema och include-traverseringsregler. Utdataformatet
harleds fran utdatasokvagen:

- `.yaml` och `.yml` genererar YAML
- `.toml` genererar TOML
- `.json` och `.json5` genererar JSON5-kompatibla mallar
- okanda eller saknade filandelse genererar YAML

Anvand `write_config_schemas` for att skapa Draft 7 JSON Schemas for
rotkonfigurationen och `x-tree-split`-markerade nastlade sektioner. De genererade schemana utelamnar
`required`-begransningar sa IDE:er kan erbjuda komplettering for partiella
konfigurationsfiler utan att rapportera saknade falt. Genererade
`*.schema.json`-filer ar bara for IDE-komplettering och grundlaggande
editor-kontroller; de avgor inte om ett konkret faltvarde ar giltigt for
programmet. Faltvardevalidering ska implementeras i kod med
`#[config(validate = Self::validate)]` och koras via `load_config` eller
`config-validate`:

```rust
use rust_config_tree::write_config_schemas;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;

    Ok(())
}
```

Markera ett nastlat falt med `#[schemars(extend("x-tree-split" = true))]` nar
det ska genereras som en egen `*.yaml`-mall och ett eget
`<section>.schema.json`-schema. Omarkerade nastlade falt stannar i
foraldramallen och foraldraschemat.

Markera ett bladfalt med `#[schemars(extend("x-env-only" = true))]` nar vardet bara ska komma fran miljovariabler. Genererade mallar och JSON Schemas utelamnar env-only-falt, och foralderobjekt som blir tomma tas bort.

For ett schema med sektionerna `server` och `log` markerade med `x-tree-split` skriver detta
`schemas/myapp.schema.json`, `schemas/server.schema.json` och
`schemas/log.schema.json`. Rotschemat innehaller bara falt som hor hemma i
rotkonfigurationsfilen, som `include` och rotens skalara falt. Det utelamnar
avsiktligt delade sektionsproperties, sa `server` och `log` kompletteras bara
nar deras egna YAML-sektionsfiler redigeras.

Anvand `write_config_templates` for att skapa en rotmall och varje mallfil som
nas fran dess include-trad:

```rust
use rust_config_tree::write_config_templates;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;

    Ok(())
}
```

Anvand `write_config_templates_with_schema` nar genererade TOML-, YAML-, JSON-
och JSON5-mallar ska binda dessa scheman for IDE-komplettering och
grundlaggande schemakontroller:

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

Rotmal for TOML och YAML binder rotschemat och kompletterar inte delade
barnsektioners falt. Delade sektionsmal for YAML binder sitt motsvarande
sektionsschema, till exempel far `log.yaml`
`# yaml-language-server: $schema=./schemas/log.schema.json`. JSON- och
JSON5-mal far ett rotfalt `$schema` som VS Code kan kanna igen. VS Code
`json.schemas` ar fortfarande en alternativ bindningsvag.

Mallgenerering valjer kalltrad i denna ordning:

- en befintlig konfigurationssokvag
- en befintlig utdatakatalog eller mallutdatasokvag
- utdatasokvagen, behandlad som ett nytt tomt malltrad

Om en kallnod saknar include-lista harleder `rust-config-tree`
barnmallfiler fran nastlade `confique`-sektioner markerade med `x-tree-split`. Med schemat ovan producerar en
tom `config.example.yaml`-kalla:

```text
config.example.yaml
server.yaml
```

Rotmallen far ett include-block for `server.yaml`. YAML-mal som mappar
till en nastlad sektion, till exempel `server.yaml`, innehaller bara den
sektionen. Ytterligare nastlade sektioner delas rekursivt bara nar de falten ocksa bar `x-tree-split`.

Overstyr `template_path_for_section` nar en sektion ska genereras pa en annan
sokvag:

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

Standardvagen for sektioner ar `<section>.yaml` for nastlade sektioner
pa toppniva. Nastlade barn placeras under foraldrafilens stam, till exempel
`trading/risk.yaml`.

## CLI-integrering

Platta ut `ConfigCommand` i programmets befintliga clap-kommandoenum for att
lagga till:

- `config-template`
- `config-schema`
- `config-validate`
- `completions`
- `install-completions`
- `uninstall-completions`

Det konsumerande programmet behaller sin egen `Parser`-typ och sin egen
kommandoenum. `rust-config-tree` bidrar bara med ateranvandbara underkommandon:

1. Lagg till `#[command(subcommand)] command: Command` i programmets parser.
2. Lagg till `#[command(flatten)] Config(ConfigCommand)` i programmets
   `Subcommand`-enum.
3. Clap expanderar de utplattade varianterna till samma underkommandoniva som
   programmets egna kommandon.
4. Matcha den varianten och anropa `handle_config_command::<Cli, AppConfig>`.

Programspecifika override-flaggor for konfiguration stannar pa programmets egen
parser. Till exempel kan `--server-port` mappa till `server.port` genom att
bygga ett nastlat `CliOverrides { server: Some(CliServerOverrides { port }) }`
-varde och sla samman det med `Serialized::defaults`.

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

`config-template --output <file-name>` skriver mallar under
`config/<root_config_name>/` med det valda filnamnet. Om en sokvag anges anvands
bara filnamnet. Om inget utdatafilnamn anges skriver det
`config/<root_config_name>/<root_config_name>.example.yaml`. Lagg till
`--schema <path>` for att binda TOML-, YAML-, JSON- och JSON5-mallar till en
genererad JSON Schema-uppsattning. JSON- och JSON5-mallar far ett `$schema`-falt
som VS Code kanner igen. Detta skriver ocksa rotschemat och sektionsscheman till
den valda schemasokvagen.

`config-schema --output <path>` skriver rotens Draft 7 JSON Schema och
sektionsscheman. Om ingen utdatasokvag anges skrivs rotschemat till
`config/<root_config_name>/<root_config_name>.schema.json`.

`config-validate` laddar hela runtime-konfigurationstradet och kor `confique`
standardvarden och validering, inklusive validatorer deklarerade med
`#[config(validate = Self::validate)]`. Anvand editorscheman for tyst
komplettering medan delade filer redigeras; anvand detta kommando for
obligatoriska falt och slutlig konfigurationsvalidering. Det skriver
`Configuration is ok` nar valideringen lyckas.

`completions <shell>` skriver kompletteringar till stdout.

`install-completions <shell>` skriver kompletteringar under anvandarens
hemkatalog och uppdaterar skalets startfil nar skalet kraver det. Bash, Elvish,
Fish, PowerShell och Zsh stods.

`uninstall-completions <shell>` tar bort den aktuella binarens completion-fil
och tar bort det hanterade shell-startblocket nar skalet anvander ett.

## Lagre niva: trad-API

Anvand `load_config_tree` nar du inte anvander `confique` eller nar du behover
direkt atkomst till traverseringsresultat:

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

Trad-API:t normaliserar sokvagar lexikalt, avvisar tomma include-sokvagar,
detekterar rekursiva include-cykler och hoppar over filer som redan laddats via
en annan include-gren.

## Licens

Licensierad enligt valfritt av:

- Apache License, Version 2.0
- MIT license

enligt ditt val.
