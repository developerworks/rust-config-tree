# rust-config-tree

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

`rust-config-tree` は、階層化された設定ファイルを使う Rust アプリケーション
向けに、設定ツリーの読み込みと CLI 補助機能を提供します。

プロジェクトマニュアル: <https://developerworks.github.io/rust-config-tree/>。
各言語のマニュアルは独立した mdBook サイトとして公開され、言語切り替え
リンクを持ちます。

主な機能:

- Figment の runtime provider を通して `confique` schema を実用可能な
  config オブジェクトへ読み込む
- `config-template`、`config-schema`、`config-validate`、`completions`、
  `install-completions`、`uninstall-completions` のコマンド処理
- エディタ補完と基本的な schema check 向けの Draft 7 root / section JSON Schema 生成
- YAML、TOML、JSON、JSON5 の設定テンプレート生成
- TOML、YAML、JSON、JSON5 template の schema 連携
- 再帰的な include 走査
- 環境変数をマージする前の `.env` 読み込み
- Figment metadata による source tracking
- `tracing` による TRACE レベルの source tracking ログ
- include を宣言したファイルからの相対 include path 解決
- 字句的な path 正規化
- include cycle 検出
- 決定的な走査順
- mirror された template target の収集
- `x-tree-split` で mark した nested schema section の YAML template 分割

アプリケーションは `confique::Config` を derive し、`ConfigSchema` を実装して
include field を公開します。

## Install

```toml
[dependencies]
rust-config-tree = "0.1"
confique = { version = "0.4", features = ["yaml", "toml", "json5"] }
figment = { version = "0.10", features = ["yaml", "toml", "json", "env"] }
schemars = { version = "1", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
clap = { version = "4", features = ["derive"] }
```

## Configuration Schema

include field はアプリケーション側の schema が所有します。
`rust-config-tree` は、中間の `confique` layer から include を取り出す小さな
adapter だけを必要とします。

```rust
use std::path::PathBuf;

use confique::Config;
use schemars::JsonSchema;
use rust_config_tree::ConfigSchema;

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(default = "paper")]
    mode: String,

    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    server: ServerConfig,
}

#[derive(Debug, Config, JsonSchema)]
struct ServerConfig {
    #[config(default = 8080)]
    port: u16,
}

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}
```

相対 include path は、それを宣言したファイルから解決されます。

```yaml
# config.yaml
include:
  - config/server.yaml

mode: shadow
```

```yaml
# config/server.yaml
server:
  port: 7777
```

最終 schema は `load_config` で読み込みます。

```rust
use rust_config_tree::load_config;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = load_config::<AppConfig>("config.yaml")?;
    println!("{config:#?}");

    Ok(())
}
```

`load_config` は root config file のディレクトリから上方向に最初の `.env`
ファイルを探して読み込み、その後 Figment に schema で宣言された環境変数を
読ませます。プロセス環境に既に存在する値は保持され、`.env` の値より優先
されます。

runtime config の読み込みは Figment が担当します。`confique` は schema
metadata、default、validation、template generation を担当します。環境変数名は
`#[config(env = "...")]` から読み取られます。loader は
`Env::split("_")` や `Env::split("__")` を使わないため、
`APP_DATABASE_POOL_SIZE` を `database.pool_size` に安全に対応付けできます。

`load_config` は command-line arguments を読みません。CLI flag は
application-specific です。CLI override が必要な場合は
`build_config_figment` の後に provider を merge し、
`load_config_from_figment` で検証します。

CLI flag 名は config path から自動生成されません。`--server-port` や
`--database-url` のような通常の application flag を使い、アプリケーションが
nested override 構造へ変換します。`CliOverrides` provider に含めた値だけが
設定を上書きします。

```rust
use figment::providers::Serialized;
use serde::Serialize;
use rust_config_tree::{build_config_figment, load_config_from_figment};

#[derive(Debug, Serialize)]
struct CliOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<String>,
}

fn load_with_cli_overrides(cli_mode: Option<String>) -> Result<AppConfig, Box<dyn std::error::Error + Send + Sync>> {
    let cli_overrides = CliOverrides {
        mode: cli_mode,
    };

    let figment = build_config_figment::<AppConfig>("config.yaml")?
        .merge(Serialized::defaults(cli_overrides));

    let config = load_config_from_figment::<AppConfig>(&figment)?;
    Ok(config)
}
```

この形で CLI override を merge した場合、runtime precedence は次の通りです。

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```

source metadata が必要な場合は `load_config_with_figment` を使います。

```rust
use rust_config_tree::load_config_with_figment;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;

    if let Some(metadata) = figment.find_metadata("mode") {
        let source = metadata.interpolate(&figment::Profile::Default, &["mode"]);
        println!("mode came from {source}");
    }

    println!("{config:#?}");

    Ok(())
}
```

loader は `tracing::trace!` で source tracking も出力します。このイベントは
アプリケーションの tracing subscriber で TRACE が有効な場合だけ生成されます。

## Template Generation

template は同じ schema と include 走査ルールから生成されます。出力 format は
output path から推定されます。

- `.yaml` と `.yml` は YAML
- `.toml` は TOML
- `.json` と `.json5` は JSON5-compatible template
- unknown extension または extension なしは YAML

`write_config_schemas` は root config と split nested section の Draft 7 JSON Schema
を生成します。生成 schema は `required` constraint を省略するため、IDE は
partial config file に補完を出しながら missing field diagnostic を出しません。
生成された `*.schema.json` は IDE 補完と基本的な editor check のためのもので、
具体的な field value が application として合法かどうかは判断しません。
field value validation は code 側で `#[config(validate = Self::validate)]` として
実装し、`load_config` または `config-validate` で実行します。

```rust
use rust_config_tree::write_config_schemas;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;

    Ok(())
}
```

Mark a nested field with `#[schemars(extend("x-tree-split" = true))]` when it
should be generated as its own `*.yaml` template and
`<section>.schema.json` schema. Unmarked nested fields stay in the parent
template and parent schema.

`#[schemars(extend("x-env-only" = true))]` を leaf field に付けると、その値は環境変数からだけ渡すものとして扱われます。生成される template と JSON Schema は env-only field を省略し、その結果空になった parent object も削除します.

`server` と `log` section を `x-tree-split` で mark した schema では
`schemas/myapp.schema.json`、`schemas/server.schema.json`、
`schemas/log.schema.json` が生成されます。root schema は root config file に
属する field のみを含み、split child section の補完は各 section YAML ファイルで
のみ有効になります。mark していない nested section は root schema に残ります。

`write_config_templates` は root template と include tree から到達できる
template file を作成します。

```rust
use rust_config_tree::write_config_templates;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;

    Ok(())
}
```

TOML / YAML / JSON / JSON5 template に schema 連携も付ける場合は
`write_config_templates_with_schema` を使います。

```rust
use rust_config_tree::write_config_templates_with_schema;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_templates_with_schema::<AppConfig>(
        "config.toml",
        "config.example.toml",
        "schemas/myapp.schema.json",
    )?;

    Ok(())
}
```

TOML / YAML の root target は root schema に bind され、split child section
field は補完しません。split section YAML target は対応する section schema に
bind されます。JSON / JSON5 target は VS Code が認識できる root `$schema`
field を受け取ります。

## CLI Integration

既存の clap command enum に `ConfigCommand` を flatten すると、次の再利用可能な
subcommand を追加できます。

- `config-template`
- `config-schema`
- `config-validate`
- `completions`
- `install-completions`
- `uninstall-completions`

`config-validate` は完全な runtime config tree を読み込み、`confique` defaults
と validation を実行します。これには
`#[config(validate = Self::validate)]` で宣言した validator も含まれます。
成功時は `Configuration is ok` を出力します。

## Lower-Level Tree API

`confique` を使わない場合、または走査結果へ直接アクセスしたい場合は
`load_config_tree` を使います。

```rust
use std::{fs, io, path::{Path, PathBuf}};

use rust_config_tree::{ConfigSource, load_config_tree};

fn load_source(path: &Path) -> io::Result<ConfigSource<String>> {
    let content = fs::read_to_string(path)?;
    let includes = content
        .lines()
        .filter_map(|line| line.strip_prefix("include: "))
        .map(PathBuf::from)
        .collect();

    Ok(ConfigSource::new(content, includes))
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let tree = load_config_tree("config.yaml", load_source)?;

    for node in tree.nodes() {
        println!("{}", node.path().display());
    }

    Ok(())
}
```

Tree API は path を字句的に正規化し、空の include path を拒否し、再帰的な
include cycle を検出し、別の include branch で既に読み込まれた file を
skip します。

## License

次のいずれかの license を選択できます。

- Apache License, Version 2.0
- MIT license
