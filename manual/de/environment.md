# Umgebungsvariablen

[English](../en/environment.html) | [中文](../zh/environment.html) | [日本語](../ja/environment.html) | [한국어](../ko/environment.html) | [Français](../fr/environment.html) | [Deutsch](environment.html) | [Español](../es/environment.html) | [Português](../pt/environment.html) | [Svenska](../sv/environment.html) | [Suomi](../fi/environment.html) | [Nederlands](../nl/environment.html)

Umgebungsvariablennamen werden im Schema mit `confique` deklariert:

```rust
#[derive(Debug, Config)]
struct DatabaseConfig {
    #[config(env = "APP_DATABASE_URL")]
    url: String,

    #[config(default = 16)]
    #[config(env = "APP_DATABASE_POOL_SIZE")]
    pool_size: u32,
}
```

`rust-config-tree` liest diese Namen aus `confique::Config::META` und baut
einen Figment-Provider, der jede Umgebungsvariable ihrem exakten Feldpfad
zuordnet.

Verwende fuer diese Crate kein trennzeichenbasiertes Figment-Environment-
Mapping:

```rust
// Do not use this pattern for rust-config-tree schemas.
Env::prefixed("APP_").split("_")
Env::prefixed("APP_").split("__")
```

`split("_")` behandelt Unterstriche als Trenner fuer verschachtelte Schluessel.
Dadurch wird aus `APP_DATABASE_POOL_SIZE` ein Pfad wie `database.pool.size`,
was mit Rust-Feldnamen wie `pool_size` kollidiert.

Mit `ConfiqueEnvProvider` ist die Zuordnung explizit:

```text
APP_DATABASE_POOL_SIZE -> database.pool_size
```

Einzelne Unterstriche bleiben Teil des Umgebungsvariablennamens. Figment raet
die Verschachtelungsregel nicht.

## Dotenv-Laden

Bevor Laufzeitprovider ausgewertet werden, sucht der Loader nach einer
`.env`-Datei, indem er vom Verzeichnis der Root-Konfigurationsdatei nach oben
laeuft.

Bereits vorhandene Prozess-Umgebungsvariablen bleiben erhalten. Werte aus
`.env` fuellen nur fehlende Umgebungsvariablen.

Beispiel:

```dotenv
APP_SERVER_PORT=9000
APP_DATABASE_POOL_SIZE=64
```

Diese Variablen ueberschreiben Konfigurationsdateiwerte, wenn das Schema
passende `#[config(env = "...")]`-Attribute deklariert.

## Werte parsen

Der Bridge-Provider laesst Figment Umgebungswerte parsen. Er ruft nicht die
`parse_env`-Hooks von `confique` auf. Lege komplexe Werte in
Konfigurationsdateien ab, ausser die Figment-Syntax fuer Umgebungswerte passt
gut zum Typ.
