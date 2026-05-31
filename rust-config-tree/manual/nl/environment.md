# Omgevingsvariabelen

[English](../en/environment.html) | [中文](../zh/environment.html) | [日本語](../ja/environment.html) | [한국어](../ko/environment.html) | [Français](../fr/environment.html) | [Deutsch](../de/environment.html) | [Español](../es/environment.html) | [Português](../pt/environment.html) | [Svenska](../sv/environment.html) | [Suomi](../fi/environment.html) | [Nederlands](environment.html)

Namen van omgevingsvariabelen worden in het schema gedeclareerd met `confique`:

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

`rust-config-tree` leest die namen uit `confique::Config::META` en bouwt een
Figment-provider die elke omgevingsvariabele naar het exacte veldpad mappt.

Gebruik geen delimiter-gebaseerde Figment-omgevingsmapping voor deze crate:

```rust
// Do not use this pattern for rust-config-tree schemas.
Env::prefixed("APP_").split("_")
Env::prefixed("APP_").split("__")
```

`split("_")` behandelt underscores als scheidingstekens voor geneste sleutels.
Daardoor wordt `APP_DATABASE_POOL_SIZE` een pad zoals `database.pool.size`, wat
botst met Rust-veldnamen zoals `pool_size`.

Met `ConfiqueEnvProvider` is deze mapping expliciet:

```text
APP_DATABASE_POOL_SIZE -> database.pool_size
```

Enkele underscores blijven onderdeel van de naam van de omgevingsvariabele.
Figment raadt de nestingregel niet.

## Dotenv laden

Voordat runtimeproviders worden geevalueerd, zoekt de loader naar een `.env`-
bestand door omhoog te lopen vanaf de directory van het rootconfiguratiebestand.

Bestaande procesomgevingsvariabelen blijven behouden. Waarden uit `.env` vullen
alleen ontbrekende omgevingsvariabelen aan.

Voorbeeld:

```dotenv
APP_SERVER_PORT=9000
APP_DATABASE_POOL_SIZE=64
```

Deze variabelen overschrijven configuratiebestandswaarden wanneer het schema
passende `#[config(env = "...")]`-attributen declareert.

## Waarden parsen

De bridgeprovider laat Figment omgevingswaarden parsen. Hij roept de
`parse_env`-hooks van `confique` niet aan. Houd complexe waarden in
configuratiebestanden tenzij de Figment-syntaxis voor omgevingswaarden goed bij
het type past.
