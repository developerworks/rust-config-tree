# IDE-Vervollstaendigung

[English](../en/ide-completions.html) | [中文](../zh/ide-completions.html) | [日本語](../ja/ide-completions.html) | [한국어](../ko/ide-completions.html) | [Français](../fr/ide-completions.html) | [Deutsch](ide-completions.html) | [Español](../es/ide-completions.html) | [Português](../pt/ide-completions.html) | [Svenska](../sv/ide-completions.html) | [Suomi](../fi/ide-completions.html) | [Nederlands](../nl/ide-completions.html)

Erzeugte JSON-Schemas koennen von TOML-, YAML-, JSON- und JSON5-
Konfigurationsdateien verwendet werden. Sie werden aus demselben Rust-Typ
erzeugt, den `confique` verwendet:

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

Erzeuge sie mit:

```rust
use rust_config_tree::write_config_schemas;

write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Dies schreibt das Root-Schema und Abschnittsschemas wie
`schemas/server.schema.json`. Erzeugte Schemas lassen `required`-
Einschraenkungen weg, damit Vervollstaendigung fuer partielle
Konfigurationsdateien ohne Fehlende-Felder-Diagnosen funktioniert. Das
Root-Schema laesst aufgeteilte Abschnittseigenschaften weg, sodass
Kindabschnitts-Vervollstaendigung nur in Dateien verfuegbar ist, die das
passende Abschnittsschema binden. Nicht markierte verschachtelte Abschnitte
bleiben im Root-Schema.

Mit `x-env-only` markierte Felder werden aus erzeugten Schemas weggelassen, sodass IDEs keine Secrets oder andere Werte vorschlagen, die nur aus Umgebungsvariablen kommen duerfen.

IDE-Schemas dienen der Vervollstaendigung und grundlegenden Editor-Pruefungen,
etwa Typ-, Enum- und Unbekannte-Eigenschaft-Pruefungen, soweit sie vom erzeugten
Schema unterstuetzt werden. Sie entscheiden nicht, ob ein konkreter Feldwert fuer
die Anwendung gueltig ist. Feldwertvalidierung muss im Code mit
`#[config(validate = Self::validate)]` implementiert und dann ueber
`load_config` oder `config-validate` ausgefuehrt werden. Pflichtfelder und die
finale Validierung der zusammengefuehrten Konfiguration verwenden ebenfalls
diese Laufzeitpfade.

## TOML

TOML-Dateien sollten das Schema mit einer `#:schema`-Direktive am Dateianfang
binden:

```toml
#:schema ./schemas/myapp.schema.json

[server]
bind = "0.0.0.0"
port = 3000
```

Verwende in TOML kein Root-Feld `$schema = "..."`. Es wird zu echten
Konfigurationsdaten und kann die Laufzeit-Deserialisierung beeinflussen.
`write_config_templates_with_schema` fuegt die `#:schema`-Direktive fuer
TOML-Vorlagen automatisch hinzu.

## YAML

YAML-Dateien sollten die YAML-Language-Server-Modeline verwenden:

```yaml
# yaml-language-server: $schema=./schemas/myapp.schema.json

server:
  bind: 0.0.0.0
  port: 3000
```

`write_config_templates_with_schema` fuegt diese Modeline fuer YAML-Vorlagen
automatisch hinzu. Aufgeteilte YAML-Vorlagen binden ihr Abschnittsschema, zum
Beispiel bindet `log.yaml` `./schemas/log.schema.json`.

## JSON

JSON- und JSON5-Dateien koennen ein Schema mit einem obersten `$schema`-Feld binden. `write_config_templates_with_schema` fuegt es fuer erzeugte JSON- und JSON5-Vorlagen automatisch hinzu:

```json
{
  "$schema": "./schemas/myapp.schema.json"
}
```

Editor-Einstellungen bleiben nuetzlich, wenn ein Projekt keine Bindung in der Datei will:

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

YAML kann ebenfalls ueber VS-Code-Einstellungen gebunden werden:

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

Das finale Layout ist:

```text
schemas/myapp.schema.json:
  Nur Felder der Root-Datei

schemas/server.schema.json:
  Schema fuer den Abschnitt server

config.toml:
  #:schema ./schemas/myapp.schema.json

config.yaml:
  # yaml-language-server: $schema=./schemas/myapp.schema.json

server.yaml:
  # yaml-language-server: $schema=./schemas/server.schema.json

config.json:
  "$schema": "./schemas/myapp.schema.json"
```

Referenzen:

- [Tombi JSON Schema](https://tombi-toml.github.io/tombi/docs/json-schema/)
- [Taplo directives](https://taplo.tamasfe.dev/configuration/directives.html)
- [YAML Language Server](https://github.com/redhat-developer/yaml-language-server)
- [VS Code JSON](https://code.visualstudio.com/Docs/languages/json)
