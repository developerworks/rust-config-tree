# Miljovariabler

[English](../en/environment.html) | [中文](../zh/environment.html) | [日本語](../ja/environment.html) | [한국어](../ko/environment.html) | [Français](../fr/environment.html) | [Deutsch](../de/environment.html) | [Español](../es/environment.html) | [Português](../pt/environment.html) | [Svenska](environment.html) | [Suomi](../fi/environment.html) | [Nederlands](../nl/environment.html)

Miljovariabelnamn deklareras i schemat med `confique`:

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

`rust-config-tree` laser dessa namn fran `confique::Config::META` och bygger en
Figment-provider som mappar varje miljovariabel till dess exakta faltvag.

Anvand inte delimiterbaserad Figment-miljomappning for denna crate:

```rust
// Do not use this pattern for rust-config-tree schemas.
Env::prefixed("APP_").split("_")
Env::prefixed("APP_").split("__")
```

`split("_")` behandlar understreck som separatorer for nastlade nycklar. Det gor
att `APP_DATABASE_POOL_SIZE` blir en vag som `database.pool.size`, vilket
krockar med Rust-faltnamn som `pool_size`.

Med `ConfiqueEnvProvider` ar mappningen explicit:

```text
APP_DATABASE_POOL_SIZE -> database.pool_size
```

Enskilda understreck fortsatter vara en del av miljovariabelnamnet. Figment
gissar ingen nastlingsregel.

## Dotenv-laddning

Innan runtime-providers utvarderas letar laddaren efter en `.env`-fil genom att
ga uppat fran rotkonfigurationsfilens katalog.

Befintliga processmiljovariabler bevaras. Varden fran `.env` fyller bara i
saknade miljovariabler.

Exempel:

```dotenv
APP_SERVER_PORT=9000
APP_DATABASE_POOL_SIZE=64
```

Dessa variabler skriver over konfigurationsfilvarden nar schemat deklarerar
matchande `#[config(env = "...")]`-attribut.

## Parsning av varden

Bryggprovidern later Figment parsa miljovarden. Den anropar inte `confique`:s
`parse_env`-hooks. Hall komplexa varden i konfigurationsfiler om inte Figments
syntax for miljovarden passar typen.
