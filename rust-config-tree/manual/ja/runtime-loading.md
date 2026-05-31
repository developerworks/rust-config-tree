# ランタイム読み込み

[English](../en/runtime-loading.html) | [中文](../zh/runtime-loading.html) | [日本語](runtime-loading.html) | [한국어](../ko/runtime-loading.html) | [Français](../fr/runtime-loading.html) | [Deutsch](../de/runtime-loading.html) | [Español](../es/runtime-loading.html) | [Português](../pt/runtime-loading.html) | [Svenska](../sv/runtime-loading.html) | [Suomi](../fi/runtime-loading.html) | [Nederlands](../nl/runtime-loading.html)

runtime loading は Figment と confique に明確に分割されています。

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

main API は次の通りです。

```rust
use rust_config_tree::load_config;

let config = load_config::<AppConfig>("config.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

application が source metadata を必要とする場合は `load_config_with_figment`
を使います。

```rust
use rust_config_tree::load_config_with_figment;

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;
# let _ = (config, figment);
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

## Loading Steps

high-level loader は次の step を実行します。

1. root config path を字句的に解決する。
2. root config directory から上方向に最初の `.env` file を探して読み込む。
3. 各 config file を partial layer として読み込み include を発見する。
4. 発見した config files から Figment graph を構築する。
5. file より高い priority で `ConfiqueEnvProvider` を merge する。
6. 必要に応じて application-specific CLI override を merge する。
7. Figment から `confique` layer を extract する。
8. `confique` code default を適用する。
9. 最終 schema を validate して構築する。

`load_config` と `load_config_with_figment` は step 1-5 と 7-9 を実行します。
step 6 は application-specific です。この crate は CLI flag と schema field の
対応を推測しません。

## File Formats

runtime file provider は config path extension から選択されます。

- `.yaml` と `.yml` は YAML。
- `.toml` は TOML。
- `.json` と `.json5` は JSON。
- unknown extension または extension なしは YAML。

template generation は引き続き confique の YAML、TOML、JSON5-compatible
template renderer を使います。

## Include Priority

high-level loader は、included file が include した file より lower priority に
なるように file provider を merge します。root config file は最も高い file
priority を持ちます。

environment variables はすべての config file より高い priority を持ちます。
`confique` default は runtime provider が値を提供しない場合だけ使われます。

`build_config_figment` の後に CLI override を merge した場合、完全な
precedence は次の通りです。

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```

command-line syntax は `rust-config-tree` が定義しません。application が
`--server-port` を parse し、その値を nested serialized provider に map すれば
`server.port` を上書きできます。`--server.port` や `a.b.c` のような syntax は、
application が実装した場合だけ存在します。

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

