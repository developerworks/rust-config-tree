# サンプル

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

これらのサンプルは、それぞれ一時的な設定ファイルを作成する小さな実行可能
プログラムです。

リポジトリ root から実行します。

```bash
cargo run --example basic_loading
cargo run --example cli_overrides -- --server-port 9000
cargo run --example config_commands -- config-template --output /tmp/config.example.yaml
cargo run --example config_commands -- config-schema --output /tmp/myapp.schema.json
cargo run --example config_commands -- config-validate
cargo run --example generate_templates
cargo run --example tree_api
```

サンプルの内容:

- `basic_loading.rs`: recursive config tree から `confique` schema を読み込む。
- `cli_overrides.rs`: application CLI flag を最優先の Figment provider として
  merge する。
- `config_commands.rs`: `ConfigCommand` を application clap CLI に flatten する。
- `generate_templates.rs`: schema から root / section JSON Schema と
  schema-bound TOML/YAML template を書き出す。
- `tree_api.rs`: format-agnostic な低レベル include tree API を使う。

