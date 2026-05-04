# CLI-integrering

[English](../en/cli.html) | [中文](../zh/cli.html) | [日本語](../ja/cli.html) | [한국어](../ko/cli.html) | [Français](../fr/cli.html) | [Deutsch](../de/cli.html) | [Español](../es/cli.html) | [Português](../pt/cli.html) | [Svenska](cli.html) | [Suomi](../fi/cli.html) | [Nederlands](../nl/cli.html)

`ConfigCommand` tillhandahaller ateranvandbara clap-underkommandon:

- `config-template`
- `config-schema`
- `config-validate`
- `completions`
- `install-completions`
- `uninstall-completions`

Dessa inbyggda underkommandon ar separata fran programspecifika
override-flaggor for konfiguration. Slå samman override-flaggor som
Figment-providers i runtime-laddningsvagen.

Override-flaggor for konfiguration forblir en del av det konsumerande
programmets CLI. Deras namn behover inte matcha punktade
konfigurationssokvagar. Till exempel kan programmet parsa `--server-port` och
mappa det till den nastlade konfigurationsnyckeln `server.port`. Endast flaggor
som programmet mappar till `CliOverrides` paverkar konfigurationsvarden.

Platta ut det i ett programs kommandoenum:

1. Behall programmets egen `Parser`-typ.
2. Behall programmets egen `Subcommand`-enum.
3. Lagg till `#[command(flatten)] Config(ConfigCommand)` i den enumen.
4. Clap expanderar de utplattade `ConfigCommand`-varianterna till samma kommandoniva som programmets egna varianter.
5. Matcha varianten `Config(command)` och skicka den till `handle_config_command`.

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

## Konfigurationsmallar

```bash
demo config-template
```

Kommandot skriver mallar under `config/<root_config_name>/`. Om `--output` far
en sokvag anvands bara filnamnet. Om inget utdatafilnamn anges skriver
kommandot `config/<root_config_name>/<root_config_name>.example.yaml`. Lagg till
`--schema schemas/myapp.schema.json` for att binda genererade TOML-, YAML-,
JSON- och JSON5-mallar till genererade JSON Schemas. Delade YAML-mallar binder
matchande sektionsschema. JSON- och JSON5-mallar far ett `$schema`-falt som
VS Code kanner igen. Kommandot skriver ocksa rot- och sektionsscheman till den
valda schemasokvagen.

```bash
demo config-template --output app_config.example.toml --schema schemas/myapp.schema.json
```

Generera rot- och sektions-JSON Schemas:

```bash
demo config-schema
```

Utan `--output` skriver `config-schema` rotschemat till
`config/<root_config_name>/<root_config_name>.schema.json`.

Validera hela runtime-konfigurationstradet:

```bash
demo config-validate
```

Genererade editorscheman undviker avsiktligt required-field-diagnostik for
delade filer. `config-validate` laddar includes, tillampar standardvarden och
kor slutlig `confique`-validering, inklusive validatorer deklarerade med
`#[config(validate = Self::validate)]`. Genererade `*.schema.json`-filer ar for
IDE-komplettering och grundlaggande editor-kontroller, inte for
faltvardelegalitet. Det skriver `Configuration is ok` nar valideringen lyckas.

## Skalkompletteringar

Skriv kompletteringar till stdout:

```bash
demo completions zsh
```

Installera kompletteringar:

```bash
demo install-completions zsh
```

Avinstallera kompletteringar:

```bash
demo uninstall-completions zsh
```

Installeraren stoder Bash, Elvish, Fish, PowerShell och Zsh. Den skriver
kompletteringsfilen under anvandarens hemkatalog och uppdaterar skalets
startfil for skal som kraver det.

Innan en befintlig skalstartfil som `~/.zshrc`, `~/.bashrc`, en Elvish
rc-fil eller en PowerShell-profil andras skriver kommandot en backup bredvid
originalfilen:

```text
<rc-file>.backup.by.<program-name>.<timestamp>
```
