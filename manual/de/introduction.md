# Einfuehrung

[English](../en/introduction.html) | [中文](../zh/introduction.html) | [日本語](../ja/introduction.html) | [한국어](../ko/introduction.html) | [Français](../fr/introduction.html) | [Deutsch](introduction.html) | [Español](../es/introduction.html) | [Português](../pt/introduction.html) | [Svenska](../sv/introduction.html) | [Suomi](../fi/introduction.html) | [Nederlands](../nl/introduction.html)

`rust-config-tree` stellt wiederverwendbares Laden von Konfigurationsbaeumen
und CLI-Helfer fuer Rust-Anwendungen bereit, die geschichtete
Konfigurationsdateien verwenden.

Die Crate ist um eine kleine Verantwortungsaufteilung herum gebaut:

- `confique` besitzt Schemadefinitionen, Code-Defaults, Validierung und
  Konfigurationsvorlagenerzeugung.
- `figment` besitzt Laufzeitladen und Laufzeit-Quellenmetadaten.
- `rust-config-tree` besitzt rekursive Include-Traversierung,
  Include-Pfadaufloesung, `.env`-Laden, Vorlagenziel-Erkennung und
  wiederverwendbare clap-Befehle.

Die Crate ist nuetzlich, wenn eine Anwendung ein natuerliches Dateilayout wie
dieses verwenden soll:

```yaml
include:
  - config/server.yaml
  - config/database.yaml

log:
  level: info
```

Jede inkludierte Datei kann dieselbe Schemaform verwenden, und relative
Include-Pfade werden von der Datei aufgeloest, die sie deklariert hat. Die
finale Konfiguration bleibt ein normales `confique`-Schema.

## Hauptfunktionen

- Rekursive Include-Traversierung mit Zyklenerkennung.
- Relative Include-Pfade werden von der deklarierenden Datei aufgeloest.
- `.env` wird geladen, bevor Umgebungsprovider ausgewertet werden.
- Schema-deklarierte Umgebungsvariablen ohne Trennzeichen-Splitting.
- Figment-Metadaten fuer Quellenverfolgung zur Laufzeit.
- Quellenverfolgungsereignisse auf TRACE-Ebene ueber `tracing`.
- Erzeugung von Draft-7-JSON-Schemas fuer Editor-Vervollstaendigung und
  grundlegende Schema-Pruefungen.
- Feldwertvalidierung im Anwendungscode mit
  `#[config(validate = Self::validate)]`, ausgefuehrt durch `load_config` oder
  `config-validate`.
- Vorlagenerzeugung fuer YAML, TOML, JSON und JSON5.
- TOML-`#:schema` und YAML-Language-Server-Modelines fuer erzeugte Vorlagen.
- Opt-in-YAML-Vorlagenaufteilung fuer mit `x-tree-split` markierte Abschnitte.
- Eingebaute clap-Unterbefehle fuer Konfigurationsvorlagen, JSON-Schema und
  Shell-Vervollstaendigungen.
- Eine untergeordnete Tree-API fuer Aufrufer, die `confique` nicht verwenden.

## Oeffentliche Einstiegspunkte

Diese APIs eignen sich fuer die meisten Anwendungen:

- `load_config::<S>(path)` laedt das finale Schema.
- `load_config_with_figment::<S>(path)` laedt das Schema und gibt den
  Figment-Graphen fuer Quellenverfolgung zurueck.
- `write_config_templates::<S>(config_path, output_path)` schreibt die
  Root-Vorlage und rekursiv entdeckte Kind-Vorlagen.
- `write_config_schemas::<S>(output_path)` schreibt Draft-7-JSON-Schemas fuer
  Root und Abschnitte.
- `handle_config_command::<Cli, S>(command, config_path)` behandelt eingebaute
  clap-Konfigurationsbefehle.

Verwende `load_config_tree`, wenn du das Traversierungsprimitiv ohne
`confique` brauchst.
