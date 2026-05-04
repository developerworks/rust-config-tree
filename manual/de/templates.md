# Vorlagenerzeugung

[English](../en/templates.html) | [中文](../zh/templates.html) | [日本語](../ja/templates.html) | [한국어](../ko/templates.html) | [Français](../fr/templates.html) | [Deutsch](templates.html) | [Español](../es/templates.html) | [Português](../pt/templates.html) | [Svenska](../sv/templates.html) | [Suomi](../fi/templates.html) | [Nederlands](../nl/templates.html)

Vorlagen werden aus demselben `confique`-Schema erzeugt, das zur Laufzeit
verwendet wird. `confique` rendert den eigentlichen Vorlageninhalt,
einschliesslich Doc-Kommentaren, Defaults, Pflichtfeldern und deklarierten
Umgebungsvariablennamen.

Verwende `write_config_templates`:

```rust
use rust_config_tree::write_config_templates;

write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Erzeuge Draft-7-JSON-Schemas fuer die Root-Konfiguration und explizit aufgeteilte verschachtelte
Abschnitte:

```rust
use rust_config_tree::write_config_schemas;

write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Mark a nested field with `#[schemars(extend("x-tree-split" = true))]` when it
should be generated as its own `*.yaml` template and
`<section>.schema.json` schema. Unmarked nested fields stay in the parent
template and parent schema.

Markiere ein Blattfeld mit `#[schemars(extend("x-env-only" = true))]`, wenn der Wert nur aus Umgebungsvariablen kommen darf. Generierte Vorlagen und JSON-Schemas lassen env-only-Felder weg, und dadurch leere Elternobjekte werden entfernt.

Erzeugte Schemas lassen `required`-Einschraenkungen weg. IDEs koennen weiterhin
Vervollstaendigung anbieten, aber partielle Dateien wie `log.yaml`
melden keine fehlenden Root-Felder. Das Root-Schema vervollstaendigt nur
Felder, die in die Root-Datei gehoeren; aufgeteilte Abschnittsfelder werden
dort weggelassen und durch ihre eigenen Abschnittsschemas vervollstaendigt.
Vorhandene Felder koennen weiterhin grundlegende Editor-Pruefungen erhalten,
etwa Typ-, Enum- und Unbekannte-Eigenschaft-Pruefungen, soweit sie vom erzeugten
Schema unterstuetzt werden. Erzeugte `*.schema.json`-Dateien entscheiden nicht,
ob ein konkreter Feldwert fuer die Anwendung gueltig ist. Feldwertvalidierung
muss im Code mit `#[config(validate = Self::validate)]` implementiert werden;
`load_config` und `config-validate` fuehren diese Laufzeitvalidierung aus.

Binde diese Schemas aus erzeugten TOML-, YAML-, JSON- und JSON5-Vorlagen:

```rust
use rust_config_tree::write_config_templates_with_schema;

write_config_templates_with_schema::<AppConfig>(
    "config.toml",
    "config.example.toml",
    "schemas/myapp.schema.json",
)?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

TOML- und YAML-Root-Vorlagen binden das Root-Schema und vervollstaendigen keine
aufgeteilten untergeordneten Abschnittsfelder. Aufgeteilte
YAML-Abschnittsvorlagen binden ihr Abschnittsschema. JSON- und JSON5-Vorlagen
erhalten ein oberstes `$schema`-Feld, das VS Code erkennen kann. VS Code
`json.schemas` bleibt als alternative Bindung moeglich.

Das Ausgabeformat wird aus dem Ausgabepfad abgeleitet:

- `.yaml` und `.yml` erzeugen YAML.
- `.toml` erzeugt TOML.
- `.json` und `.json5` erzeugen JSON5-kompatible Vorlagen.
- unbekannte oder fehlende Erweiterungen erzeugen YAML.

## Schema-Bindungen

Mit einem Schemapfad `schemas/myapp.schema.json` verwenden erzeugte
Root-Vorlagen:

```toml
#:schema ./schemas/myapp.schema.json
```

```yaml
# yaml-language-server: $schema=./schemas/myapp.schema.json
```

Erzeugte Abschnittsvorlagen binden Abschnittsschemas:

```yaml
# log.yaml
# yaml-language-server: $schema=./schemas/log.schema.json
```

Erzeugte JSON- und JSON5-Vorlagen schreiben ein oberstes `$schema`-Feld, das VS Code erkennt. Editor-Einstellungen bleiben optional:

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

## Auswahl der Vorlagenquelle

Die Vorlagenerzeugung waehlt ihren Quellbaum in dieser Reihenfolge:

1. Vorhandener Konfigurationspfad.
2. Vorhandener Ausgabe-Vorlagenpfad.
3. Ausgabepfad, behandelt als neuer leerer Vorlagenbaum.

So kann ein Projekt Vorlagen aus aktuellen Konfigurationsdateien aktualisieren,
ein vorhandenes Vorlagenset aktualisieren oder ein neues Vorlagenset nur aus
dem Schema erzeugen.

## Gespiegelte Include-Baeume

Wenn die Quelldatei Includes deklariert, spiegeln erzeugte Vorlagen diese
Include-Pfade unter dem Ausgabeverzeichnis.

```yaml
# config.yaml
include:
  - server.yaml
```

Das Erzeugen von `config.example.yaml` schreibt:

```text
config.example.yaml
server.yaml
```

Relative Include-Ziele werden unter dem Elternverzeichnis der Ausgabedatei
gespiegelt. Absolute Include-Ziele bleiben absolut.

## Opt-in-Abschnittsaufteilung

Wenn eine Quelldatei keine Includes hat, kann die Crate Include-Ziele aus
mit `x-tree-split` markierten verschachtelten Schemaabschnitten ableiten. Fuer ein Schema mit einem markierten Abschnitt
`server` kann eine leere Root-Vorlagenquelle Folgendes erzeugen:

```text
config.example.yaml
server.yaml
```

Die Root-Vorlage erhaelt einen Include-Block, und `server.yaml` enthaelt
nur den Abschnitt `server`. Verschachtelte Abschnitte werden nur rekursiv aufgeteilt, wenn diese Felder ebenfalls `x-tree-split` tragen.
