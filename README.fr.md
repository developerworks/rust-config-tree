# rust-config-tree

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

`rust-config-tree` fournit le chargement d'arbres de configuration et des
assistants CLI pour les applications Rust qui utilisent des fichiers de
configuration en couches.

Manuel du projet : <https://developerworks.github.io/rust-config-tree/>. Les
manuels en plusieurs langues sont publies comme sites mdBook independants avec
des liens de changement de langue.

Il gere :

- le chargement d'un schema `confique` dans un objet de configuration
  directement utilisable via des fournisseurs Figment d'execution ;
- les gestionnaires de commandes `config-template`, `config-schema`,
  `config-validate`, `completions`, `install-completions` et
  `uninstall-completions` ;
- la generation de schemas JSON Draft 7 pour la racine et les sections, pour la
  completion et les controles de schema de base dans l'editeur ;
- la generation de modeles de configuration YAML, TOML, JSON et JSON5 ;
- les directives de schema pour les modeles TOML et YAML sans ajouter de champs
  d'execution ;
- la traversee recursive des inclusions ;
- le chargement de `.env` avant la fusion des valeurs d'environnement ;
- le suivi des sources via les metadonnees Figment ;
- les journaux de suivi des sources au niveau TRACE via `tracing` ;
- la resolution des chemins d'inclusion relatifs depuis le fichier qui les
  declare ;
- la normalisation lexicale des chemins ;
- la detection des cycles d'inclusion ;
- un ordre de traversee deterministe ;
- la collecte miroir des cibles de modeles ;
- decoupage opt-in des modeles YAML pour les sections imbriquees
  marquees `x-tree-split`.

Les applications fournissent leur schema en derivant `confique::Config` et en
implementant `ConfigSchema` pour exposer le champ d'inclusion du schema.

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

## Schema de configuration

Le schema de votre application possede le champ d'inclusion. `rust-config-tree`
n'a besoin que d'un petit adaptateur qui extrait les inclusions depuis la couche
intermediaire `confique`.

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

Les chemins d'inclusion relatifs sont resolus depuis le fichier qui les
declare :

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

Chargez le schema final avec `load_config` :

```rust
use rust_config_tree::load_config;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = load_config::<AppConfig>("config.yaml")?;
    println!("{config:#?}");

    Ok(())
}
```

`load_config` charge le premier fichier `.env` trouve en remontant depuis le
repertoire du fichier de configuration racine avant de demander a Figment de
lire les variables d'environnement declarees par le schema. Les valeurs deja
presentes dans l'environnement du processus sont conservees et ont priorite sur
les valeurs de `.env`.

Le chargement d'execution est effectue via Figment. `confique` reste
responsable des metadonnees de schema, des valeurs par defaut, de la validation
et de la generation de modeles. Les noms de variables d'environnement sont lus
depuis `#[config(env = "...")]` ; le chargeur n'utilise pas `Env::split("_")`
ni `Env::split("__")`, donc une variable comme `APP_DATABASE_POOL_SIZE` peut
correspondre a un champ nomme `database.pool_size`.

`load_config` ne lit pas les arguments de ligne de commande, car les drapeaux
CLI sont propres a chaque application. Ajoutez les remplacements CLI en
fusionnant un fournisseur apres `build_config_figment`, puis validez avec
`load_config_from_figment` :

Les noms de drapeaux CLI ne sont pas derives des chemins de configuration.
Utilisez des drapeaux d'application normaux comme `--server-port` ou
`--database-url` ; ne vous appuyez pas sur `--server.port` ou `a.b.c` sauf si
l'application implemente deliberement cet analyseur. La forme imbriquee
serialisee du remplacement decide quelle cle de configuration est remplacee.

Seules les valeurs representees dans le fournisseur `CliOverrides` de
l'application peuvent remplacer la configuration. C'est prevu pour les
parametres d'execution souvent ajustes, lorsqu'il vaut mieux changer un drapeau
pour une execution que modifier un fichier de configuration. Gardez la
configuration stable dans les fichiers et n'exposez comme drapeaux CLI que les
remplacements temporaires voulus.

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

Avec des remplacements CLI fusionnes ainsi, la priorite d'execution est :

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```

Utilisez `load_config_with_figment` lorsque l'appelant a besoin des metadonnees
de source :

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

Le chargeur emet aussi le suivi des sources de configuration avec
`tracing::trace!`. Ces evenements ne sont produits que lorsque TRACE est active
par le subscriber `tracing` de l'application. Si `tracing` est initialise apres
le chargement de la configuration, appelez `trace_config_sources::<AppConfig>(&figment)`
apres avoir installe le subscriber.

## Generation de modeles

Les modeles sont rendus avec le meme schema et les memes regles de traversee
d'inclusions. Le format de sortie est deduit du chemin de sortie :

- `.yaml` et `.yml` generent du YAML ;
- `.toml` genere du TOML ;
- `.json` et `.json5` generent des modeles compatibles JSON5 ;
- les extensions inconnues ou absentes generent du YAML.

Utilisez `write_config_schemas` pour creer des schemas JSON Draft 7 pour la
configuration racine et les sections imbriquees decoupees. Les schemas generes omettent
les contraintes `required` afin que les IDE puissent proposer la completion pour
des fichiers de configuration partiels sans signaler de champs manquants. Les
fichiers `*.schema.json` generes servent uniquement a la completion IDE et aux
controles d'editeur de base ; ils ne decident pas si une valeur de champ concrete
est valide pour l'application. La validation de valeur doit etre implementee dans
le code avec `#[config(validate = Self::validate)]`, puis executee par
`load_config` ou `config-validate` :

```rust
use rust_config_tree::write_config_schemas;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;

    Ok(())
}
```

Mark a nested field with `#[schemars(extend("x-tree-split" = true))]` when it
should be generated as its own `config/*.yaml` template and
`schemas/*.schema.json` schema. Unmarked nested fields stay in the parent
template and parent schema.

Marquez un champ feuille avec `#[schemars(extend("x-env-only" = true))]` lorsque la valeur doit venir uniquement de variables d environnement. Les modeles generes et les schemas JSON omettent les champs env-only, et les objets parents devenus vides sont supprimes.

Pour un schema avec les sections `server` et `log` marquees `x-tree-split`, cela ecrit
`schemas/myapp.schema.json`, `schemas/server.schema.json` et
`schemas/log.schema.json`. Le schema racine ne contient que les champs qui
appartiennent au fichier racine, comme `include` et les champs scalaires racine.
Il omet intentionnellement les proprietes des sections decoupees, donc `server`
et `log` ne sont completes que lors de l'edition de leurs propres fichiers YAML
de section. Les sections imbriquees non marquees restent dans le schema racine.

Utilisez `write_config_templates` pour creer un modele racine et chaque fichier
modele accessible depuis son arbre d'inclusion :

```rust
use rust_config_tree::write_config_templates;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;

    Ok(())
}
```

Utilisez `write_config_templates_with_schema` lorsque les modeles TOML et YAML
generes doivent lier ces schemas pour la completion et les controles de schema
de base dans l'IDE :

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

Les cibles TOML/YAML racine lient le schema racine et ne completent pas les
champs des sections enfants. Les cibles YAML de section separee lient leur
schema de section correspondant ; par exemple `config/log.yaml` recoit
`# yaml-language-server: $schema=../schemas/log.schema.json`. Les cibles JSON et
JSON5 ne recoivent volontairement pas de champ `$schema` ; liez-les avec des
parametres d'editeur comme `json.schemas` dans VS Code.

La generation de modeles choisit son arbre source dans cet ordre :

- un chemin de configuration existant ;
- un chemin de modele de sortie existant ;
- le chemin de sortie, traite comme un nouvel arbre de modeles vide.

Si un noeud source n'a pas de liste d'inclusions, `rust-config-tree` derive les
fichiers modeles enfants depuis les sections `confique` imbriquees marquees `x-tree-split`. Avec le
schema ci-dessus, une source `config.example.yaml` vide produit :

```text
config.example.yaml
config/server.yaml
```

Le modele racine recoit un bloc d'inclusion pour `config/server.yaml`. Les
cibles YAML qui correspondent a une section imbriquee, comme
`config/server.yaml`, ne contiennent que cette section. Les sections encore plus
imbriquees ne sont separees recursivement que lorsque ces champs portent aussi `x-tree-split`.

Remplacez `template_path_for_section` lorsqu'une section doit etre generee a un
autre chemin :

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

Le chemin de section par defaut est `config/<section>.yaml` pour les sections
imbriquees de premier niveau. Les enfants imbriques sont places sous le stem du
fichier parent, par exemple `config/trading/risk.yaml`.

## Integration CLI

Aplatissez `ConfigCommand` dans votre enum de commandes clap existante pour
ajouter :

- `config-template`
- `config-schema`
- `config-validate`
- `completions`
- `install-completions`
- `uninstall-completions`

L'application consommatrice garde son propre type `Parser` et sa propre enum de
commandes. `rust-config-tree` ne fournit que des sous-commandes reutilisables :

1. Ajoutez `#[command(subcommand)] command: Command` au parser de l'application.
2. Ajoutez `#[command(flatten)] Config(ConfigCommand)` a l'enum `Subcommand` de
   l'application.
3. Clap developpe les variantes aplaties au meme niveau de sous-commandes que
   les commandes propres a l'application.
4. Faites correspondre cette variante et appelez
   `handle_config_command::<Cli, AppConfig>`.

Les drapeaux de remplacement propres a l'application restent sur son propre
parser. Par exemple, `--server-port` peut correspondre a `server.port` en
construisant une valeur imbriquee
`CliOverrides { server: Some(CliServerOverrides { port }) }` et en la
fusionnant avec `Serialized::defaults`.

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

`config-template --output <file-name>` ecrit les modeles sous
`config/<root_config_name>/` avec le nom de fichier choisi. Si un chemin est
fourni, seul son nom de fichier est utilise. Si aucun nom de fichier de sortie
n'est fourni, il ecrit
`config/<root_config_name>/<root_config_name>.example.yaml`. Ajoutez
`--schema <path>` pour lier les modeles TOML et YAML a un ensemble de schemas
JSON generes sans ajouter de champ `$schema` d'execution. Cela ecrit aussi le
schema racine et les schemas de section au chemin de schema choisi.

`config-schema --output <path>` ecrit le schema JSON Draft 7 racine et les
schemas de section. Les sections imbriquees non marquees restent dans le schema racine. Si aucun chemin de sortie n'est fourni, le schema racine est
ecrit dans `config/<root_config_name>/<root_config_name>.schema.json`.

`config-validate` charge l'arbre complet de configuration d'execution et lance
les valeurs par defaut et la validation `confique`, y compris les validateurs
declares avec `#[config(validate = Self::validate)]`. Utilisez les schemas
d'editeur pour une completion non bruyante pendant l'edition de fichiers
separes ; utilisez cette commande pour les champs obligatoires et la validation
finale de la configuration. Elle affiche `Configuration is ok` lorsque la
validation reussit.

`completions <shell>` imprime les completions sur stdout.

`install-completions <shell>` ecrit les completions sous le repertoire home de
l'utilisateur et met a jour le fichier de demarrage du shell lorsque le shell le
requiert. Bash, Elvish, Fish, PowerShell et Zsh sont pris en charge.

`uninstall-completions <shell>` supprime le fichier de completion du binaire
courant et supprime le bloc de demarrage gere quand ce shell en utilise un.

## API d'arbre de plus bas niveau

Utilisez `load_config_tree` lorsque vous n'utilisez pas `confique` ou lorsque
vous avez besoin d'un acces direct aux resultats de traversee :

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

L'API d'arbre normalise les chemins lexicalement, rejette les chemins
d'inclusion vides, detecte les cycles d'inclusion recursifs et ignore les
fichiers deja charges par une autre branche d'inclusion.

## Licence

Sous licence, au choix :

- Apache License, Version 2.0
- MIT license
