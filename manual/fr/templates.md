# Generation de modeles

[English](../en/templates.html) | [中文](../zh/templates.html) | [日本語](../ja/templates.html) | [한국어](../ko/templates.html) | [Français](templates.html) | [Deutsch](../de/templates.html) | [Español](../es/templates.html) | [Português](../pt/templates.html) | [Svenska](../sv/templates.html) | [Suomi](../fi/templates.html) | [Nederlands](../nl/templates.html)

Les modeles sont generes depuis le meme schema `confique` que celui utilise a
l'execution. `confique` rend le contenu reel du modele, y compris les
commentaires de documentation, les valeurs par defaut, les champs obligatoires
et les noms de variables d'environnement declares.

Utilisez `write_config_templates` :

```rust
use rust_config_tree::write_config_templates;

write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Generez des schemas JSON Draft 7 pour la configuration racine et les sections
imbriquees :

```rust
use rust_config_tree::write_config_schemas;

write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Mark a nested field with `#[schemars(extend("x-tree-split" = true))]` when it
should be generated as its own `*.yaml` template and
`<section>.schema.json` schema. Unmarked nested fields stay in the parent
template and parent schema.

Marquez un champ feuille avec `#[schemars(extend("x-env-only" = true))]` lorsque la valeur doit venir uniquement de variables d environnement. Les modeles generes et les schemas JSON omettent les champs env-only, et les objets parents devenus vides sont supprimes.

Les schemas generes omettent les contraintes `required`. Les IDE peuvent
toujours proposer la completion, mais les fichiers partiels comme
`log.yaml` ne signalent pas de champs racine manquants. Le schema racine
ne complete que les champs qui appartiennent au fichier racine ; les champs de
sections imbriquees y sont omis et sont completes par leurs propres schemas de
section. Les champs presents peuvent encore recevoir des controles d'editeur de
base, comme les types, les enums et les proprietes inconnues pris en charge par
le schema genere. Les `*.schema.json` generes ne decident pas si une valeur de
champ concrete est valide pour l'application. La validation de valeur doit etre
implementee dans le code avec `#[config(validate = Self::validate)]` ; `load_config`
et `config-validate` executent cette validation d'execution.

Liez ces schemas depuis les modeles TOML, YAML, JSON et JSON5 generes :

```rust
use rust_config_tree::write_config_templates_with_schema;

write_config_templates_with_schema::<AppConfig>(
    "config.toml",
    "config.example.toml",
    "schemas/myapp.schema.json",
)?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Les modeles racine TOML et YAML lient le schema racine et ne completent pas les
champs des sections enfants. Les modeles YAML de section separee lient leur
schema de section. Les modeles JSON et JSON5 recoivent un champ racine
`$schema` que VS Code peut reconnaitre. VS Code `json.schemas` reste une autre
facon de lier le schema.

Le format de sortie est deduit du chemin de sortie :

- `.yaml` et `.yml` generent du YAML.
- `.toml` genere du TOML.
- `.json` et `.json5` generent des modeles compatibles JSON5.
- les extensions inconnues ou absentes generent du YAML.

## Liaisons de schema

Avec un chemin de schema `schemas/myapp.schema.json`, les modeles racine generes
utilisent :

```toml
#:schema ./schemas/myapp.schema.json
```

```yaml
# yaml-language-server: $schema=./schemas/myapp.schema.json
```

Les modeles de section generes lient les schemas de section :

```yaml
# log.yaml
# yaml-language-server: $schema=./schemas/log.schema.json
```

Les modeles JSON et JSON5 generes ecrivent un champ racine `$schema` reconnu
par VS Code. Les parametres d'editeur restent optionnels :

```json
{
  "json.schemas": [
    {
      "fileMatch": [
        "/config.json",
        "/config.*.json"
      ],
      "url": "./schemas/myapp.schema.json"
    }
  ]
}
```

## Selection de la source des modeles

La generation de modeles choisit son arbre source dans cet ordre :

1. Chemin de configuration existant.
2. Chemin de modele de sortie existant.
3. Chemin de sortie traite comme nouvel arbre de modeles vide.

Cela permet a un projet de mettre a jour les modeles depuis les fichiers de
configuration actuels, de mettre a jour un ensemble de modeles existant ou de
creer un nouvel ensemble de modeles uniquement depuis le schema.

## Arbres d'inclusion miroirs

Si le fichier source declare des inclusions, les modeles generes reproduisent
ces chemins d'inclusion sous le repertoire de sortie.

```yaml
# config.yaml
include:
  - server.yaml
```

Generer `config.example.yaml` ecrit :

```text
config.example.yaml
server.yaml
```

Les cibles d'inclusion relatives sont reproduites sous le repertoire parent du
fichier de sortie. Les cibles d'inclusion absolues restent absolues.

## Decoupage opt-in des sections

Lorsqu'un fichier source n'a pas d'inclusions, la crate peut deriver les cibles
d'inclusion depuis les sections de schema imbriquees marquees `x-tree-split`. Pour un schema avec une
section `server` marquee, une source de modele racine vide peut produire :

```text
config.example.yaml
server.yaml
```

Le modele racine recoit un bloc d'inclusion, et `server.yaml` ne contient
que la section `server`. Les sections imbriquees ne sont decoupees recursivement que lorsque ces champs portent aussi `x-tree-split`.
