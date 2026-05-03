# CLI-integratie

[English](../en/cli.html) | [中文](../zh/cli.html) | [日本語](../ja/cli.html) | [한국어](../ko/cli.html) | [Français](../fr/cli.html) | [Deutsch](../de/cli.html) | [Español](../es/cli.html) | [Português](../pt/cli.html) | [Svenska](../sv/cli.html) | [Suomi](../fi/cli.html) | [Nederlands](cli.html)

`ConfigCommand` biedt herbruikbare clap-subcommands:

- `config-template`
- `config-schema`
- `config-validate`
- `completions`
- `install-completions`
- `uninstall-completions`

Deze ingebouwde subcommands staan los van toepassingsspecifieke configuratie-
overridevlaggen. Voeg configuratie-overridevlaggen als Figment-providers samen
in het runtime-laadpad.

Configuratie-overridevlaggen blijven onderdeel van de CLI van de consumerende
toepassing. Hun namen hoeven niet overeen te komen met gestippelde
configuratiepaden. De toepassing kan bijvoorbeeld `--server-port` parsen en
naar de geneste configuratiesleutel `server.port` mappen. Alleen vlaggen die de
toepassing in `CliOverrides` mappt, beinvloeden configuratiewaarden.

Flatten het in een toepassingsopdrachtenenum:

1. Behoud het eigen `Parser`-type van de toepassing.
2. Behoud de eigen `Subcommand`-enum van de toepassing.
3. Voeg `#[command(flatten)] Config(ConfigCommand)` aan die enum toe.
4. Clap breidt de geflattende `ConfigCommand`-varianten uit naar hetzelfde
   opdracht niveau als de eigen varianten van de toepassing.
5. Match de variant `Config(command)` en geef die door aan `handle_config_command`.

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

## Configuratiesjablonen

```bash
demo config-template --output config.example.yaml
```

Als geen uitvoerpad is opgegeven, schrijft de opdracht `config.example.yaml` in
de huidige directory. Voeg `--schema schemas/myapp.schema.json` toe om
gegenereerde TOML- en YAML-sjablonen te koppelen aan gegenereerde JSON Schemas.
Gesplitste YAML-sjablonen koppelen het passende sectieschema. De opdracht
schrijft ook het root- en de sectieschema's naar het gekozen schemapad.

```bash
demo config-template --output config.example.toml --schema schemas/myapp.schema.json
```

Genereer root- en sectie-JSON Schemas:

```bash
demo config-schema --output schemas/myapp.schema.json
```

Valideer de volledige runtimeconfiguratieboom:

```bash
demo config-validate
```

Gegenereerde editorschema's vermijden bewust diagnostics voor verplichte velden
in gesplitste bestanden. `config-validate` laadt includes, past defaults toe en
voert uiteindelijke `confique`-validatie uit. Het print `Configuration is ok`
wanneer de validatie slaagt.

## Shellcompletions

Print completions naar stdout:

```bash
demo completions zsh
```

Installeer completions:

```bash
demo install-completions zsh
```

Verwijder completions:

```bash
demo uninstall-completions zsh
```

De installer ondersteunt Bash, Elvish, Fish, PowerShell en Zsh. Hij schrijft
het completionbestand onder de home-directory van de gebruiker en werkt het
shellstartbestand bij voor shells die dat vereisen.

Voordat een bestaand shellstartbestand zoals `~/.zshrc`, `~/.bashrc`, een
Elvish rc-bestand of een PowerShell-profiel wordt gewijzigd, schrijft de
opdracht een backup naast het originele bestand:

```text
<rc-file>.backup.by.<program-name>.<timestamp>
```
