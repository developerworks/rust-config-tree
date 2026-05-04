# Integration CLI

[English](../en/cli.html) | [中文](../zh/cli.html) | [日本語](../ja/cli.html) | [한국어](../ko/cli.html) | [Français](cli.html) | [Deutsch](../de/cli.html) | [Español](../es/cli.html) | [Português](../pt/cli.html) | [Svenska](../sv/cli.html) | [Suomi](../fi/cli.html) | [Nederlands](../nl/cli.html)

`ConfigCommand` fournit des sous-commandes clap reutilisables :

- `config-template`
- `config-schema`
- `config-validate`
- `completions`
- `install-completions`
- `uninstall-completions`

Ces sous-commandes integrees sont separees des drapeaux de remplacement de
configuration propres a l'application. Fusionnez les drapeaux de remplacement
comme fournisseurs Figment dans le chemin de chargement d'execution.

Les drapeaux de remplacement de configuration restent dans la CLI de
l'application consommatrice. Leurs noms n'ont pas besoin de correspondre aux
chemins de configuration avec points. Par exemple, l'application peut analyser
`--server-port` et le mapper a la cle de configuration imbriquee `server.port`.
Seuls les drapeaux que l'application mappe dans `CliOverrides` affectent les
valeurs de configuration.

Aplatissez-le dans une enum de commandes d'application :

1. Gardez le type `Parser` propre a l'application.
2. Gardez l'enum `Subcommand` propre a l'application.
3. Ajoutez `#[command(flatten)] Config(ConfigCommand)` a cette enum.
4. Clap developpe les variantes `ConfigCommand` aplaties au meme niveau de
   commande que les variantes propres a l'application.
5. Faites correspondre la variante `Config(command)` et passez-la a
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

## Modeles de configuration

```bash
demo config-template
```

La commande ecrit les modeles sous `config/<root_config_name>/`. Si `--output`
recoit un chemin, seul le nom de fichier est utilise. Si aucun nom de fichier de
sortie n'est fourni, la commande ecrit
`config/<root_config_name>/<root_config_name>.example.yaml`. Ajoutez
`--schema schemas/myapp.schema.json` pour lier les modeles TOML, YAML, JSON et
JSON5 generes aux schemas JSON generes. Les modeles YAML separes lient le
schema de section correspondant. Les modeles JSON et JSON5 recoivent un champ
`$schema` reconnu par VS Code. La commande ecrit aussi les schemas racine et de
section au chemin de schema choisi.

```bash
demo config-template --output app_config.example.toml --schema schemas/myapp.schema.json
```

Generer les schemas JSON racine et de section :

```bash
demo config-schema
```

Sans `--output`, `config-schema` ecrit le schema racine dans
`config/<root_config_name>/<root_config_name>.schema.json`.

Valider l'arbre complet de configuration d'execution :

```bash
demo config-validate
```

Les schemas d'editeur generes evitent intentionnellement les diagnostics de
champs obligatoires pour les fichiers separes. `config-validate` charge les
inclusions, applique les valeurs par defaut et lance la validation finale
`confique`, y compris les validateurs declares avec
`#[config(validate = Self::validate)]`. Les `*.schema.json` generes restent
reserves a la completion IDE et aux controles d'editeur de base, pas a la
validation de legalite des valeurs. Elle affiche `Configuration is ok` lorsque
la validation reussit.

## Completions shell

Imprimer les completions sur stdout :

```bash
demo completions zsh
```

Installer les completions :

```bash
demo install-completions zsh
```

Desinstaller les completions :

```bash
demo uninstall-completions zsh
```

L'installateur prend en charge Bash, Elvish, Fish, PowerShell et Zsh. Il ecrit
le fichier de completion sous le repertoire home de l'utilisateur et met a jour
le fichier de demarrage du shell pour les shells qui l'exigent.

Avant de modifier un fichier de demarrage shell existant comme `~/.zshrc`,
`~/.bashrc`, un fichier rc Elvish ou un profil PowerShell, la commande ecrit
une sauvegarde a cote du fichier original :

```text
<rc-file>.backup.by.<program-name>.<timestamp>
```
