# Introduction

[English](../en/introduction.html) | [中文](../zh/introduction.html) | [日本語](../ja/introduction.html) | [한국어](../ko/introduction.html) | [Français](introduction.html) | [Deutsch](../de/introduction.html) | [Español](../es/introduction.html) | [Português](../pt/introduction.html) | [Svenska](../sv/introduction.html) | [Suomi](../fi/introduction.html) | [Nederlands](../nl/introduction.html)

`rust-config-tree` fournit un chargement reutilisable d'arbres de configuration
et des assistants CLI pour les applications Rust qui utilisent des fichiers de
configuration en couches.

La crate est concue autour d'une petite separation des responsabilites :

- `confique` possede les definitions de schema, les valeurs par defaut du code,
  la validation et la generation de modeles de configuration.
- `figment` possede le chargement d'execution et les metadonnees de source
  d'execution.
- `rust-config-tree` possede la traversee recursive des inclusions, la
  resolution des chemins d'inclusion, le chargement de `.env`, la decouverte des
  cibles de modeles et les commandes clap reutilisables.

La crate est utile lorsqu'une application veut une disposition naturelle des
fichiers de configuration comme celle-ci :

```yaml
include:
  - config/server.yaml
  - config/database.yaml

log:
  level: info
```

Chaque fichier inclus peut utiliser la meme forme de schema, et les chemins
d'inclusion relatifs sont resolus depuis le fichier qui les a declares. La
configuration finale reste une valeur de schema `confique` normale.

## Fonctionnalites principales

- Traversee recursive des inclusions avec detection des cycles.
- Chemins d'inclusion relatifs resolus depuis le fichier declarant.
- Chargement de `.env` avant l'evaluation des fournisseurs d'environnement.
- Variables d'environnement declarees par le schema sans separation par
  delimiteur.
- Metadonnees Figment pour le suivi des sources d'execution.
- Evenements de suivi des sources au niveau TRACE via `tracing`.
- Generation de schemas JSON Draft 7 pour la completion et les controles de
  schema de base dans l'editeur.
- Validation des valeurs de champ dans le code applicatif avec
  `#[config(validate = Self::validate)]`, executee par `load_config` ou
  `config-validate`.
- Generation de modeles YAML, TOML, JSON et JSON5.
- Directives de schema TOML `#:schema`, modelines YAML Language Server et
  champs JSON/JSON5 `$schema` pour les modeles generes.
- Decoupage opt-in des modeles YAML pour les sections marquees `x-tree-split`.
- Sous-commandes clap integrees pour les modeles de configuration, les schemas
  JSON et les completions shell.
- API d'arbre de plus bas niveau pour les appelants qui n'utilisent pas
  `confique`.

## Points d'entree publics

Utilisez ces API pour la plupart des applications :

- `load_config::<S>(path)` charge le schema final.
- `load_config_with_figment::<S>(path)` charge le schema et renvoie le graphe
  Figment utilise pour le suivi des sources.
- `write_config_templates::<S>(config_path, output_path)` ecrit le modele racine
  et les modeles enfants decouverts recursivement.
- `write_config_schemas::<S>(output_path)` ecrit les schemas JSON Draft 7
  racine et de section.
- `handle_config_command::<Cli, S>(command, config_path)` gere les commandes de
  configuration clap integrees.

Utilisez `load_config_tree` lorsque vous avez besoin de la primitive de
traversee sans `confique`.
