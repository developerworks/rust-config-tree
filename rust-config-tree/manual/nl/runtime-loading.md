# Runtime laden

[English](../en/runtime-loading.html) | [中文](../zh/runtime-loading.html) | [日本語](../ja/runtime-loading.html) | [한국어](../ko/runtime-loading.html) | [Français](../fr/runtime-loading.html) | [Deutsch](../de/runtime-loading.html) | [Español](../es/runtime-loading.html) | [Português](../pt/runtime-loading.html) | [Svenska](../sv/runtime-loading.html) | [Suomi](../fi/runtime-loading.html) | [Nederlands](runtime-loading.html)

Runtime laden is bewust verdeeld tussen Figment en confique:

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

De hoofd-API is:

```rust
use rust_config_tree::load_config;

let config = load_config::<AppConfig>("config.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Gebruik `load_config_with_figment` wanneer de toepassing bronmetadata nodig heeft:

```rust
use rust_config_tree::load_config_with_figment;

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;
# let _ = (config, figment);
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

## Laadstappen

De high-level loader voert deze stappen uit:

1. Los het rootconfiguratiepad lexicaal op.
2. Laad het eerste `.env`-bestand dat wordt gevonden door omhoog te lopen vanaf
   de rootconfiguratiedirectory.
3. Laad elk configuratiebestand als gedeeltelijke laag om includes te ontdekken.
4. Bouw een Figment-grafiek uit de ontdekte configuratiebestanden.
5. Voeg de `ConfiqueEnvProvider` samen met hogere prioriteit dan bestanden.
6. Voeg eventueel toepassingsspecifieke CLI-overrides samen.
7. Extraheer een `confique`-laag uit Figment.
8. Pas `confique`-codestandaarden toe.
9. Valideer en bouw het uiteindelijke schema.

`load_config` en `load_config_with_figment` voeren stappen 1-5 en 7-9 uit.
Stap 6 is toepassingsspecifiek omdat deze crate niet kan afleiden hoe een
CLI-vlag naar een schemaveld mappt.

## Bestandsformaten

De runtimebestandsprovider wordt gekozen uit de extensie van het configuratiepad:

- `.yaml` en `.yml` gebruiken YAML.
- `.toml` gebruikt TOML.
- `.json` en `.json5` gebruiken JSON.
- onbekende of ontbrekende extensies gebruiken YAML.

Sjabloongeneratie gebruikt nog steeds de template-renderers van confique voor
YAML, TOML en JSON5-compatibele uitvoer.

## Include-prioriteit

De high-level loader voegt bestandsproviders zo samen dat geinclude bestanden
lagere prioriteit hebben dan het bestand dat ze include. Het rootconfiguratie-
bestand heeft de hoogste bestandsprioriteit.

Omgevingsvariabelen hebben hogere prioriteit dan alle configuratiebestanden.
`confique`-defaults worden alleen gebruikt voor waarden die niet door
runtimeproviders worden geleverd.

Wanneer CLI-overrides na `build_config_figment` worden samengevoegd, is de
volledige prioriteit:

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```

De commandoregelsyntaxis wordt niet door `rust-config-tree` gedefinieerd. Een
vlag zoals `--server-port` kan `server.port` overschrijven als de toepassing die
geparsede waarde in een geneste geserialiseerde provider mappt. Een gestippelde
syntaxis zoals `--server.port` of `a.b.c` bestaat alleen als de toepassing die
implementeert.

Dit betekent dat CLI-prioriteit alleen geldt voor sleutels die aanwezig zijn in
de overrideprovider van de toepassing. Gebruik dit voor operationele waarden die
vaak voor een enkele run worden gewijzigd. Laat duurzame configuratie in
bestanden staan.

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
