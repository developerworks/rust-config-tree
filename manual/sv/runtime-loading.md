# Runtime-laddning

[English](../en/runtime-loading.html) | [中文](../zh/runtime-loading.html) | [日本語](../ja/runtime-loading.html) | [한국어](../ko/runtime-loading.html) | [Français](../fr/runtime-loading.html) | [Deutsch](../de/runtime-loading.html) | [Español](../es/runtime-loading.html) | [Português](../pt/runtime-loading.html) | [Svenska](runtime-loading.html) | [Suomi](../fi/runtime-loading.html) | [Nederlands](../nl/runtime-loading.html)

Runtime-laddning ar avsiktligt uppdelad mellan Figment och confique:

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

Huvud-API:t ar:

```rust
use rust_config_tree::load_config;

let config = load_config::<AppConfig>("config.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Anvand `load_config_with_figment` nar programmet behover kallmetadata:

```rust
use rust_config_tree::load_config_with_figment;

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;
# let _ = (config, figment);
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

## Laddningssteg

Hog-niva-laddaren gor dessa steg:

1. Los rotkonfigurationssokvagen lexikalt.
2. Ladda den forsta `.env`-filen som hittas genom att ga uppat fran rotkonfigurationskatalogen.
3. Ladda varje konfigurationsfil som ett partiellt lager for att upptacka includes.
4. Bygg en Figment-graf fran de upptackta konfigurationsfilerna.
5. Slå samman `ConfiqueEnvProvider` med hogre prioritet an filer.
6. Slå eventuellt samman programspecifika CLI-overrides.
7. Extrahera ett `confique`-lager fran Figment.
8. Applicera `confique`-kodstandardvarden.
9. Validera och skapa det slutliga schemat.

`load_config` och `load_config_with_figment` utfor steg 1-5 och 7-9. Steg 6 ar
programspecifikt eftersom denna crate inte kan avgora hur en CLI-flagga mappar
till ett schemafalt.

## Filformat

Runtime-filprovidern valjs fran konfigurationssokvagens filandelse:

- `.yaml` och `.yml` anvander YAML.
- `.toml` anvander TOML.
- `.json` och `.json5` anvander JSON.
- okanda eller saknade filandelser anvander YAML.

Mallgenerering anvander fortfarande confiques mallrenderare for YAML, TOML och
JSON5-kompatibel utdata.

## Include-prioritet

Hog-niva-laddaren slar samman filproviders sa inkluderade filer har lagre
prioritet an filen som inkluderade dem. Rotkonfigurationsfilen har hogsta
filprioritet.

Miljovariabler har hogre prioritet an alla konfigurationsfiler. `confique`
standardvarden anvands bara for varden som inte tillhandahalls av
runtime-providers.

Nar CLI-overrides slas samman efter `build_config_figment` ar full prioritet:

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```

Kommandoradssyntaxen definieras inte av `rust-config-tree`. En flagga som
`--server-port` kan skriva over `server.port` om programmet mappar det parsade
vardet till en nastlad serialiserad provider. En punktad syntax som
`--server.port` eller `a.b.c` finns bara om programmet implementerar den.

Det betyder att CLI-prioritet bara galler nycklar som finns i programmets
override-provider. Anvand det for operativa varden som ofta andras for en enda
korning. Lamna varaktig konfiguration i filer.

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
