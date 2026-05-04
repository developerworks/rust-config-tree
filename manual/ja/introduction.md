# 概要

[English](../en/introduction.html) | [中文](../zh/introduction.html) | [日本語](introduction.html) | [한국어](../ko/introduction.html) | [Français](../fr/introduction.html) | [Deutsch](../de/introduction.html) | [Español](../es/introduction.html) | [Português](../pt/introduction.html) | [Svenska](../sv/introduction.html) | [Suomi](../fi/introduction.html) | [Nederlands](../nl/introduction.html)

`rust-config-tree` は、階層化された設定ファイルを使う Rust アプリケーション
向けに、再利用可能な設定ツリー読み込み機能と CLI 補助機能を提供します。

この crate は責務を小さく分けています。

- `confique` は schema 定義、code default、validation、config template
  generation を担当します。
- `figment` は runtime loading と runtime source metadata を担当します。
- `rust-config-tree` は recursive include traversal、include path resolution、
  `.env` loading、template target discovery、reusable clap command を担当します。

たとえば、次のような自然な設定ファイル layout を扱うときに便利です。

```yaml
include:
  - config/server.yaml
  - config/database.yaml

log:
  level: info
```

include された各 file は同じ schema shape を使えます。相対 include path は、
それを宣言した file から解決されます。最終的な config は通常の `confique`
schema value です。

## 主な機能

- recursive include traversal と cycle detection。
- 宣言元 file からの相対 include path resolution。
- environment provider 評価前の `.env` loading。
- delimiter splitting を使わない schema-declared environment variables。
- runtime source tracking 向け Figment metadata。
- `tracing` による TRACE-level source tracking event。
- editor completion と基本的な schema check 向け Draft 7 JSON Schema generation。
- application code で `#[config(validate = Self::validate)]` として実装し、
  `load_config` または `config-validate` で実行する field value validation。
- YAML、TOML、JSON、JSON5 template generation。
- generated TOML template の `#:schema` と YAML Language Server modeline。
- `x-tree-split` で mark した nested section の YAML template splitting。
- config template、JSON Schema、shell completion 向け built-in clap subcommands。
- `confique` を使わない caller 向けの lower-level tree API。

## 主な入口

多くの application では次の API を使います。

- `load_config::<S>(path)` は最終 schema を読み込みます。
- `load_config_with_figment::<S>(path)` は schema を読み込み、source tracking
  に使う Figment graph も返します。
- `write_config_templates::<S>(config_path, output_path)` は root template と
  recursively discovered child template を書き出します。
- `write_config_schemas::<S>(output_path)` は root / section Draft 7 JSON
  Schema を書き出します。
- `handle_config_command::<Cli, S>(command, config_path)` は built-in clap config
  command を処理します。

`confique` なしで traversal primitive だけが必要な場合は `load_config_tree`
を使います。
