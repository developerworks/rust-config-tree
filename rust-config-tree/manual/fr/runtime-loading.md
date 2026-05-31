# Chargement d'execution

[English](../en/runtime-loading.html) | [中文](../zh/runtime-loading.html) | [日本語](../ja/runtime-loading.html) | [한국어](../ko/runtime-loading.html) | [Français](runtime-loading.html) | [Deutsch](../de/runtime-loading.html) | [Español](../es/runtime-loading.html) | [Português](../pt/runtime-loading.html) | [Svenska](../sv/runtime-loading.html) | [Suomi](../fi/runtime-loading.html) | [Nederlands](../nl/runtime-loading.html)

Le chargement d'execution est volontairement separe entre Figment et confique :

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

L'API principale est :

```rust
use rust_config_tree::load_config;

let config = load_config::<AppConfig>("config.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Utilisez `load_config_with_figment` lorsque l'application a besoin des
metadonnees de source :

```rust
use rust_config_tree::load_config_with_figment;

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;
# let _ = (config, figment);
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

## Etapes de chargement

Le chargeur de haut niveau effectue ces etapes :

1. Resoudre lexicalement le chemin de configuration racine.
2. Charger le premier fichier `.env` trouve en remontant depuis le repertoire de
   configuration racine.
3. Charger chaque fichier de configuration comme couche partielle pour decouvrir
   les inclusions.
4. Construire un graphe Figment depuis les fichiers de configuration decouverts.
5. Fusionner `ConfiqueEnvProvider` avec une priorite superieure aux fichiers.
6. Fusionner eventuellement les remplacements CLI propres a l'application.
7. Extraire une couche `confique` depuis Figment.
8. Appliquer les valeurs par defaut du code `confique`.
9. Valider et construire le schema final.

`load_config` et `load_config_with_figment` effectuent les etapes 1-5 et 7-9.
L'etape 6 est propre a l'application, car cette crate ne peut pas deduire
comment un drapeau CLI correspond a un champ de schema.

## Formats de fichier

Le fournisseur de fichier d'execution est choisi depuis l'extension du chemin de
configuration :

- `.yaml` et `.yml` utilisent YAML.
- `.toml` utilise TOML.
- `.json` et `.json5` utilisent JSON.
- les extensions inconnues ou absentes utilisent YAML.

La generation de modeles utilise toujours les renderers de modeles `confique`
pour les sorties YAML, TOML et compatibles JSON5.

## Priorite des inclusions

Le chargeur de haut niveau fusionne les fournisseurs de fichiers de sorte que
les fichiers inclus aient une priorite plus faible que le fichier qui les a
inclus. Le fichier de configuration racine a la priorite de fichier la plus
elevee.

Les variables d'environnement ont une priorite superieure a tous les fichiers de
configuration. Les valeurs par defaut `confique` ne sont utilisees que pour les
valeurs non fournies par les fournisseurs d'execution.

Lorsque des remplacements CLI sont fusionnes apres `build_config_figment`, la
priorite complete est :

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```

La syntaxe de ligne de commande n'est pas definie par `rust-config-tree`. Un
drapeau comme `--server-port` peut remplacer `server.port` si l'application
mappe cette valeur analysee dans un fournisseur serialise imbrique. Une syntaxe
avec points comme `--server.port` ou `a.b.c` n'existe que si l'application
l'implemente.

Cela signifie que la priorite CLI ne s'applique qu'aux cles presentes dans le
fournisseur de remplacement de l'application. Utilisez-la pour les valeurs
operationnelles qui changent souvent pour une seule execution. Laissez la
configuration durable dans les fichiers.

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

