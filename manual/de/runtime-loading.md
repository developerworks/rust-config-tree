# Laden zur Laufzeit

[English](../en/runtime-loading.html) | [中文](../zh/runtime-loading.html) | [日本語](../ja/runtime-loading.html) | [한국어](../ko/runtime-loading.html) | [Français](../fr/runtime-loading.html) | [Deutsch](runtime-loading.html) | [Español](../es/runtime-loading.html) | [Português](../pt/runtime-loading.html) | [Svenska](../sv/runtime-loading.html) | [Suomi](../fi/runtime-loading.html) | [Nederlands](../nl/runtime-loading.html)

Das Laden zur Laufzeit ist bewusst zwischen Figment und confique aufgeteilt:

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

Die Haupt-API ist:

```rust
use rust_config_tree::load_config;

let config = load_config::<AppConfig>("config.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Verwende `load_config_with_figment`, wenn die Anwendung Quellenmetadaten
braucht:

```rust
use rust_config_tree::load_config_with_figment;

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;
# let _ = (config, figment);
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

## Ladeschritte

Der High-Level-Loader fuehrt diese Schritte aus:

1. Root-Konfigurationspfad lexikalisch aufloesen.
2. Die erste `.env`-Datei laden, die beim Aufwaertslaufen ab dem
   Root-Konfigurationsverzeichnis gefunden wird.
3. Jede Konfigurationsdatei als Teilschicht laden, um Includes zu entdecken.
4. Einen Figment-Graphen aus den entdeckten Konfigurationsdateien bauen.
5. Den `ConfiqueEnvProvider` mit hoeherer Prioritaet als Dateien
   zusammenfuehren.
6. Optional anwendungsspezifische CLI-Ueberschreibungen zusammenfuehren.
7. Eine `confique`-Schicht aus Figment extrahieren.
8. `confique`-Code-Defaults anwenden.
9. Das finale Schema validieren und konstruieren.

`load_config` und `load_config_with_figment` fuehren die Schritte 1-5 und 7-9
aus. Schritt 6 ist anwendungsspezifisch, weil diese Crate nicht ableiten kann,
wie ein CLI-Flag auf ein Schemafeld abgebildet wird.

## Dateiformate

Der Laufzeit-Dateiprovider wird aus der Erweiterung des Konfigurationspfads
gewaehlt:

- `.yaml` und `.yml` verwenden YAML.
- `.toml` verwendet TOML.
- `.json` und `.json5` verwenden JSON.
- unbekannte oder fehlende Erweiterungen verwenden YAML.

Die Vorlagenerzeugung verwendet weiterhin die Template-Renderer von confique
fuer YAML, TOML und JSON5-kompatible Ausgabe.

## Include-Prioritaet

Der High-Level-Loader fuehrt Dateiprovider so zusammen, dass inkludierte Dateien
niedrigere Prioritaet haben als die Datei, die sie inkludiert. Die Root-
Konfigurationsdatei hat die hoechste Dateiprioritaet.

Umgebungsvariablen haben hoehere Prioritaet als alle Konfigurationsdateien.
`confique`-Defaults werden nur fuer Werte verwendet, die nicht von
Laufzeitprovidern geliefert werden.

Wenn CLI-Ueberschreibungen nach `build_config_figment` zusammengefuehrt werden,
gilt diese vollstaendige Prioritaet:

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```

Die Kommandozeilensyntax wird nicht von `rust-config-tree` definiert. Ein Flag
wie `--server-port` kann `server.port` ueberschreiben, wenn die Anwendung den
geparsten Wert in einen verschachtelten serialisierten Provider abbildet. Eine
Syntax mit Punkten wie `--server.port` oder `a.b.c` existiert nur, wenn die
Anwendung sie implementiert.

Das bedeutet: CLI-Prioritaet gilt nur fuer Schluessel, die im
Ueberschreibungsprovider der Anwendung vorhanden sind. Nutze sie fuer operative
Werte, die haeufig fuer einen einzelnen Lauf geaendert werden. Dauerhafte
Konfiguration bleibt in Dateien.

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
