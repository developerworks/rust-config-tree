# Completions IDE

[English](../en/ide-completions.html) | [中文](../zh/ide-completions.html) | [日本語](../ja/ide-completions.html) | [한국어](../ko/ide-completions.html) | [Français](ide-completions.html) | [Deutsch](../de/ide-completions.html) | [Español](../es/ide-completions.html) | [Português](../pt/ide-completions.html) | [Svenska](../sv/ide-completions.html) | [Suomi](../fi/ide-completions.html) | [Nederlands](../nl/ide-completions.html)

Les schemas JSON generes peuvent etre utilises par les fichiers de configuration
TOML, YAML, JSON et JSON5. Ils sont generes depuis le meme type Rust que celui
utilise par `confique` :

```rust
use confique::Config;
use schemars::JsonSchema;

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    server: ServerConfig,
}
```

Generez-les avec :

```rust
use rust_config_tree::write_config_schemas;

write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Cela ecrit le schema racine et les schemas de section comme
`schemas/server.schema.json`. Les schemas generes omettent les contraintes
`required` afin que la completion fonctionne pour les fichiers de configuration
partiels sans diagnostics de champs manquants. Le schema racine omet les
proprietes de sections imbriquees, donc la completion des sections enfants
n'est disponible que dans les fichiers qui lient le schema de section
correspondant.

Les champs marques `x-env-only` sont omis des schemas generes, donc les IDE ne suggerent pas les secrets ou autres valeurs qui doivent venir uniquement de variables d environnement.

Les schemas IDE servent a la completion et aux controles d'editeur de base,
comme les types, les enums et les controles de proprietes inconnues pris en
charge par le schema genere. Ils ne decident pas si une valeur de champ concrete
est valide pour l'application. La validation de valeur doit etre implementee
dans le code avec `#[config(validate = Self::validate)]`, puis executee par
`load_config` ou `config-validate`. Les champs obligatoires et la validation
finale de la configuration fusionnee utilisent aussi ces chemins d'execution.

## TOML

Les fichiers TOML doivent lier le schema avec une directive `#:schema` en haut
du fichier :

```toml
#:schema ./schemas/myapp.schema.json

[server]
bind = "0.0.0.0"
port = 3000
```

N'utilisez pas de champ racine `$schema = "..."` dans TOML. Il devient une
donnee de configuration reelle et peut affecter la deserialisation d'execution.
`write_config_templates_with_schema` ajoute automatiquement la directive
`#:schema` pour les modeles TOML.

## YAML

Les fichiers YAML doivent utiliser la modeline YAML Language Server :

```yaml
# yaml-language-server: $schema=./schemas/myapp.schema.json

server:
  bind: 0.0.0.0
  port: 3000
```

`write_config_templates_with_schema` ajoute automatiquement cette modeline pour
les modeles YAML. Les modeles YAML separes lient leur schema de section, par
exemple `log.yaml` lie `./schemas/log.schema.json`.

## JSON

Les fichiers JSON et JSON5 peuvent lier un schema avec un champ racine
`$schema`. `write_config_templates_with_schema` l'ajoute automatiquement aux
modeles JSON et JSON5 generes :

```json
{
  "$schema": "./schemas/myapp.schema.json"
}
```

Les parametres de l'editeur restent utiles si un projet ne veut pas de liaison
dans le fichier :

```json
{
  "json.schemas": [
    {
      "fileMatch": [
        "/config.json",
        "/config.*.json",
        "/deploy/*.json"
      ],
      "url": "./schemas/myapp.schema.json"
    }
  ]
}
```

YAML peut aussi etre lie via les parametres VS Code :

```json
{
  "yaml.schemas": {
    "./schemas/myapp.schema.json": [
      "config.yaml",
      "config.*.yaml",
      "deploy/*.yaml"
    ]
  }
}
```

La disposition finale est :

```text
schemas/myapp.schema.json:
  Champs du fichier racine uniquement

schemas/server.schema.json:
  Schema de la section server

config.toml:
  #:schema ./schemas/myapp.schema.json

config.yaml:
  # yaml-language-server: $schema=./schemas/myapp.schema.json

server.yaml:
  # yaml-language-server: $schema=./schemas/server.schema.json

config.json:
  "$schema": "./schemas/myapp.schema.json"
```

References :

- [Tombi JSON Schema](https://tombi-toml.github.io/tombi/docs/json-schema/)
- [Taplo directives](https://taplo.tamasfe.dev/configuration/directives.html)
- [YAML Language Server](https://github.com/redhat-developer/yaml-language-server)
- [VS Code JSON](https://code.visualstudio.com/Docs/languages/json)
