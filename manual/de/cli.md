# CLI-Integration

[English](../en/cli.html) | [中文](../zh/cli.html) | [日本語](../ja/cli.html) | [한국어](../ko/cli.html) | [Français](../fr/cli.html) | [Deutsch](cli.html) | [Español](../es/cli.html) | [Português](../pt/cli.html) | [Svenska](../sv/cli.html) | [Suomi](../fi/cli.html) | [Nederlands](../nl/cli.html)

`ConfigCommand` stellt wiederverwendbare clap-Unterbefehle bereit:

- `config-template`
- `config-schema`
- `config-validate`
- `completions`
- `install-completions`
- `uninstall-completions`

Diese eingebauten Unterbefehle sind von anwendungsspezifischen Flags fuer
Konfigurationsueberschreibungen getrennt. Fuehre solche Flags im
Laufzeit-Ladepfad als Figment-Provider zusammen.

Konfigurations-Ueberschreibungsflags bleiben Teil der CLI der konsumierenden
Anwendung. Ihre Namen muessen nicht mit gepunkteten Konfigurationspfaden
uebereinstimmen. Zum Beispiel kann die Anwendung `--server-port` parsen und auf
den verschachtelten Konfigurationsschluessel `server.port` abbilden. Nur Flags,
die die Anwendung in `CliOverrides` abbildet, beeinflussen Konfigurationswerte.

Fuege es flach in ein Befehls-Enum der Anwendung ein:

1. Behalte den eigenen `Parser`-Typ der Anwendung.
2. Behalte das eigene `Subcommand`-Enum der Anwendung.
3. Fuege `#[command(flatten)] Config(ConfigCommand)` zu diesem Enum hinzu.
4. Clap erweitert die flachen `ConfigCommand`-Varianten auf dieselbe
   Befehlsebene wie die eigenen Varianten der Anwendung.
5. Verarbeite die Variante `Config(command)` und uebergib sie an
   `handle_config_command`.

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

## Konfigurationsvorlagen

```bash
demo config-template --output app_config.example.yaml
```

Der Befehl schreibt Vorlagen unter `config/<root_config_name>/`. Wenn `--output`
einen Pfad erhaelt, wird nur der Dateiname verwendet. Wenn kein
Ausgabe-Dateiname angegeben wird, schreibt der Befehl
`config/<root_config_name>/<root_config_name>.example.yaml`. Fuege
`--schema schemas/myapp.schema.json` hinzu, um erzeugte TOML- und YAML-Vorlagen
an erzeugte JSON-Schemas zu binden. Aufgeteilte YAML-Vorlagen binden das
passende Abschnittsschema. Der Befehl schreibt ausserdem Root- und
Abschnittsschemas an den gewaehlten Schemapfad.

```bash
demo config-template --output app_config.example.toml --schema schemas/myapp.schema.json
```

Root- und Abschnitts-JSON-Schemas erzeugen:

```bash
demo config-schema --output schemas/myapp.schema.json
```

Ohne `--output` schreibt `config-schema` das Root-Schema nach
`config/<root_config_name>/<root_config_name>.schema.json`.

Den vollstaendigen Laufzeit-Konfigurationsbaum validieren:

```bash
demo config-validate
```

Erzeugte Editor-Schemas vermeiden bewusst Pflichtfeld-Diagnosen fuer
aufgeteilte Dateien. `config-validate` laedt Includes, wendet Defaults an und
fuehrt die finale `confique`-Validierung aus, einschliesslich Validatoren aus
`#[config(validate = Self::validate)]`. Erzeugte `*.schema.json`-Dateien bleiben
fuer IDE-Vervollstaendigung und grundlegende Editor-Pruefungen gedacht, nicht
fuer Feldwertlegalitaet. Bei erfolgreicher Validierung gibt es
`Configuration is ok` aus.

## Shell-Vervollstaendigungen

Vervollstaendigungen nach stdout ausgeben:

```bash
demo completions zsh
```

Vervollstaendigungen installieren:

```bash
demo install-completions zsh
```

Vervollstaendigungen deinstallieren:

```bash
demo uninstall-completions zsh
```

Der Installer unterstuetzt Bash, Elvish, Fish, PowerShell und Zsh. Er schreibt
die Vervollstaendigungsdatei unter das Home-Verzeichnis des Benutzers und
aktualisiert die Shell-Startdatei fuer Shells, die dies benoetigen.

Bevor eine vorhandene Shell-Startdatei wie `~/.zshrc`, `~/.bashrc`, eine
Elvish-rc-Datei oder ein PowerShell-Profil geaendert wird, schreibt der Befehl
ein Backup neben die Originaldatei:

```text
<rc-file>.backup.by.<program-name>.<timestamp>
```
