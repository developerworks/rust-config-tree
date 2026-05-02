# rust-config-tree

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

`rust-config-tree` stellt das Laden von Konfigurationsbaeumen und CLI-Helfer
fuer Rust-Anwendungen bereit, die geschichtete Konfigurationsdateien verwenden.

Projekthandbuch: <https://developerworks.github.io/rust-config-tree/>. Die
Handbuecher werden als eigenstaendige mdBook-Sites mit Sprachumschaltern
veroeffentlicht.

Es unterstuetzt:

- Laden eines `confique`-Schemas ueber Figment-Laufzeitprovider in ein direkt
  nutzbares Konfigurationsobjekt
- Handler fuer die Befehle `config-template`, `completions` und
  `install-completions`
- Erzeugung von Draft-7-JSON-Schemas fuer Root- und Abschnittsschemas zur
  Editor-Vervollstaendigung und Validierung
- Erzeugung von Konfigurationsvorlagen fuer YAML, TOML, JSON und JSON5
- Schema-Direktiven fuer TOML- und YAML-Vorlagen ohne zusaetzliche
  Laufzeitfelder
- rekursive Include-Traversierung
- Laden von `.env`, bevor Umgebungswerte zusammengefuehrt werden
- Quellenverfolgung ueber Figment-Metadaten
- Quellenverfolgungslogs auf TRACE-Ebene ueber `tracing`
- relative Include-Pfade, aufgeloest von der Datei, die sie deklariert
- lexikalische Pfadnormalisierung
- Erkennung von Include-Zyklen
- deterministische Traversierungsreihenfolge
- gespiegelte Sammlung von Vorlagenzielen
- automatische YAML-Vorlagenaufteilung fuer verschachtelte Schemaabschnitte

Anwendungen stellen ihr Schema bereit, indem sie `confique::Config` ableiten
und `ConfigSchema` implementieren, um das Include-Feld des Schemas offenzulegen.

## Installation

```toml
[dependencies]
rust-config-tree = "0.1"
confique = { version = "0.4", features = ["yaml", "toml", "json5"] }
figment = { version = "0.10", features = ["yaml", "env"] }
schemars = { version = "1", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
clap = { version = "4", features = ["derive"] }
```

## Konfigurationsschema

Das Anwendungsschema besitzt das Include-Feld. `rust-config-tree` benoetigt nur
einen kleinen Adapter, der Includes aus der zwischengeschalteten
`confique`-Schicht extrahiert.

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

Relative Include-Pfade werden von der Datei aufgeloest, die sie deklariert:

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

Das finale Schema wird mit `load_config` geladen:

```rust
use rust_config_tree::load_config;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = load_config::<AppConfig>("config.yaml")?;
    println!("{config:#?}");

    Ok(())
}
```

`load_config` laedt die erste `.env`-Datei, die beim Aufwaertslaufen ab dem
Verzeichnis der Root-Konfigurationsdatei gefunden wird, bevor Figment
schema-deklarierte Umgebungsvariablen liest. Bereits im Prozess vorhandene
Umgebungswerte bleiben erhalten und haben Vorrang vor `.env`-Werten.

Das Laden zur Laufzeit erfolgt ueber Figment. `confique` bleibt fuer
Schema-Metadaten, Defaults, Validierung und Vorlagenerzeugung verantwortlich.
Umgebungsvariablennamen werden aus `#[config(env = "...")]` gelesen; der Loader
verwendet weder `Env::split("_")` noch `Env::split("__")`, sodass eine Variable
wie `APP_DATABASE_POOL_SIZE` einem Feld `database.pool_size` zugeordnet werden
kann.

`load_config` liest keine Kommandozeilenargumente, weil CLI-Flags
anwendungsspezifisch sind. Fuege CLI-Ueberschreibungen hinzu, indem du nach
`build_config_figment` einen Provider zusammenfuehrst und danach mit
`load_config_from_figment` validierst.

CLI-Flag-Namen werden nicht aus Konfigurationspfaden abgeleitet. Verwende
normale Anwendungsflags wie `--server-port` oder `--database-url`; verlaess
dich nicht auf `--server.port` oder `a.b.c`, ausser die Anwendung implementiert
diesen Parser bewusst. Die verschachtelte serialisierte Ueberschreibungsform
entscheidet, welcher Konfigurationsschluessel ueberschrieben wird.

Nur Werte, die im `CliOverrides`-Provider der Anwendung dargestellt sind,
koennen Konfiguration ueberschreiben. Das ist fuer haeufig angepasste
Laufzeitparameter gedacht, bei denen ein Flag fuer einen einzelnen Lauf besser
ist als das Bearbeiten einer Konfigurationsdatei. Stabile Konfiguration gehoert
in Dateien; nur bewusste temporaere Ueberschreibungen sollten als CLI-Flags
offengelegt werden.

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

Mit so zusammengefuehrten CLI-Ueberschreibungen gilt diese Prioritaet:

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```

Verwende `load_config_with_figment`, wenn der Aufrufer Quellenmetadaten
benoetigt:

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

Der Loader gibt ausserdem Konfigurations-Quellenverfolgung mit
`tracing::trace!` aus. Diese Ereignisse entstehen nur, wenn TRACE im
`tracing`-Subscriber der Anwendung aktiviert ist. Wird `tracing` erst nach dem
Laden der Konfiguration initialisiert, rufe nach der Installation des
Subscribers `trace_config_sources::<AppConfig>(&figment)` auf.

## Vorlagenerzeugung

Vorlagen werden mit demselben Schema und denselben Include-Traversierungsregeln
gerendert. Das Ausgabeformat wird aus dem Ausgabepfad abgeleitet:

- `.yaml` und `.yml` erzeugen YAML
- `.toml` erzeugt TOML
- `.json` und `.json5` erzeugen JSON5-kompatible Vorlagen
- unbekannte oder fehlende Erweiterungen erzeugen YAML

Verwende `write_config_schemas`, um Draft-7-JSON-Schemas fuer die
Root-Konfiguration und verschachtelte Abschnitte zu erzeugen. Die erzeugten
Schemas lassen `required`-Einschraenkungen weg, damit IDEs Vervollstaendigung
fuer partielle Konfigurationsdateien anbieten koennen, ohne fehlende Felder zu
melden:

```rust
use rust_config_tree::write_config_schemas;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;

    Ok(())
}
```

Bei einem Schema mit den Abschnitten `server` und `log` schreibt dies
`schemas/myapp.schema.json`, `schemas/server.schema.json` und
`schemas/log.schema.json`. Das Root-Schema enthaelt nur Felder, die in die
Root-Konfigurationsdatei gehoeren, etwa `include` und skalare Root-Felder. Es
laesst verschachtelte Abschnittseigenschaften bewusst weg, sodass `server` und
`log` nur beim Bearbeiten ihrer eigenen Abschnitts-YAML-Dateien vervollstaendigt
werden.

Verwende `write_config_templates`, um eine Root-Vorlage und jede ueber ihren
Include-Baum erreichbare Vorlagendatei zu erzeugen:

```rust
use rust_config_tree::write_config_templates;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;

    Ok(())
}
```

Verwende `write_config_templates_with_schema`, wenn erzeugte TOML- und
YAML-Vorlagen diese Schemas fuer IDE-Vervollstaendigung und Validierung binden
sollen:

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

Root-Ziele fuer TOML/YAML binden das Root-Schema und vervollstaendigen keine
untergeordneten Abschnittsfelder. Aufgeteilte YAML-Abschnittsziele binden ihr
passendes Abschnittsschema, zum Beispiel erhaelt `config/log.yaml`
`# yaml-language-server: $schema=../schemas/log.schema.json`. JSON- und
JSON5-Ziele erhalten bewusst kein `$schema`-Feld; binde sie ueber
Editor-Einstellungen wie VS Code `json.schemas`.

Die Vorlagenerzeugung waehlt den Quellbaum in dieser Reihenfolge:

- ein vorhandener Konfigurationspfad
- ein vorhandener Ausgabe-Vorlagenpfad
- der Ausgabepfad, behandelt als neuer leerer Vorlagenbaum

Wenn ein Quellknoten keine Include-Liste hat, leitet `rust-config-tree`
Kind-Vorlagendateien aus verschachtelten `confique`-Abschnitten ab. Mit dem
obigen Schema erzeugt eine leere Quelle `config.example.yaml`:

```text
config.example.yaml
config/server.yaml
```

Die Root-Vorlage erhaelt einen Include-Block fuer `config/server.yaml`.
YAML-Ziele, die einem verschachtelten Abschnitt entsprechen, etwa
`config/server.yaml`, enthalten nur diesen Abschnitt. Weitere verschachtelte
Abschnitte werden ebenso rekursiv aufgeteilt.

Ueberschreibe `template_path_for_section`, wenn ein Abschnitt an einem anderen
Pfad erzeugt werden soll:

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

Der Standardpfad fuer Abschnitte ist `config/<section>.yaml` fuer
verschachtelte Top-Level-Abschnitte. Verschachtelte Kinder werden unter dem
Dateistamm ihres Elternteils abgelegt, zum Beispiel `config/trading/risk.yaml`.

## CLI-Integration

Fuege `ConfigCommand` flach in das vorhandene clap-Befehls-Enum ein, um diese
Befehle hinzuzufuegen:

- `config-template`
- `config-schema`
- `config-validate`
- `completions`
- `install-completions`

Die konsumierende Anwendung behaelt ihren eigenen `Parser`-Typ und ihr eigenes
Befehls-Enum. `rust-config-tree` steuert nur wiederverwendbare Unterbefehle
bei:

1. Fuege `#[command(subcommand)] command: Command` zum Parser der Anwendung
   hinzu.
2. Fuege `#[command(flatten)] Config(ConfigCommand)` zum `Subcommand`-Enum der
   Anwendung hinzu.
3. Clap erweitert die flachen Varianten auf dieselbe Unterbefehlsebene wie die
   eigenen Befehle der Anwendung.
4. Verarbeite diese Variante und rufe `handle_config_command::<Cli, AppConfig>`
   auf.

Anwendungsspezifische Flags fuer Konfigurationsueberschreibungen bleiben auf
dem eigenen Parser der Anwendung. Zum Beispiel kann `--server-port` auf
`server.port` abgebildet werden, indem ein verschachtelter Wert
`CliOverrides { server: Some(CliServerOverrides { port }) }` gebaut und mit
`Serialized::defaults` zusammengefuehrt wird.

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

`config-template --output <path>` schreibt Vorlagen an den gewaehlten Pfad. Wird
kein Ausgabepfad angegeben, schreibt der Befehl `config.example.yaml` in das
aktuelle Verzeichnis. Fuege `--schema <path>` hinzu, um TOML- und YAML-Vorlagen
an ein erzeugtes JSON-Schema-Set zu binden, ohne ein Laufzeitfeld `$schema`
hinzuzufuegen. Dabei werden auch das Root-Schema und Abschnittsschemas an den
gewaehlten Schemapfad geschrieben.

`config-schema --output <path>` schreibt das Root-Draft-7-JSON-Schema und
Abschnittsschemas. Wird kein Ausgabepfad angegeben, wird das Root-Schema nach
`schemas/config.schema.json` geschrieben.

`config-validate` laedt den vollstaendigen Laufzeit-Konfigurationsbaum und
fuehrt `confique`-Defaults und Validierung aus. Verwende Editor-Schemas fuer
rauscharme Vervollstaendigung beim Bearbeiten aufgeteilter Dateien; verwende
diesen Befehl fuer Pflichtfelder und finale Konfigurationsvalidierung. Bei
erfolgreicher Validierung gibt er `Configuration is ok` aus.

`completions <shell>` schreibt Vervollstaendigungen nach stdout.

`install-completions <shell>` schreibt Vervollstaendigungen unter das
Home-Verzeichnis des Benutzers und aktualisiert die Shell-Startdatei, wenn die
Shell dies benoetigt. Bash, Elvish, Fish, PowerShell und Zsh werden
unterstuetzt.

## Untergeordnete Tree-API

Verwende `load_config_tree`, wenn du `confique` nicht verwendest oder direkten
Zugriff auf Traversierungsergebnisse brauchst:

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

Die Tree-API normalisiert Pfade lexikalisch, weist leere Include-Pfade zurueck,
erkennt rekursive Include-Zyklen und ueberspringt Dateien, die bereits ueber
einen anderen Include-Zweig geladen wurden.

## Lizenz

Lizenziert wahlweise unter:

- Apache License, Version 2.0
- MIT license

nach deiner Wahl.
