# テンプレート生成

[English](../en/templates.html) | [中文](../zh/templates.html) | [日本語](templates.html) | [한국어](../ko/templates.html) | [Français](../fr/templates.html) | [Deutsch](../de/templates.html) | [Español](../es/templates.html) | [Português](../pt/templates.html) | [Svenska](../sv/templates.html) | [Suomi](../fi/templates.html) | [Nederlands](../nl/templates.html)

template は runtime で使う同じ `confique` schema から生成されます。実際の
template content は `confique` が render し、doc comment、default、required
field、declared environment variable name を含みます。

`write_config_templates` を使います。

```rust
use rust_config_tree::write_config_templates;

write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

root config と split nested section の Draft 7 JSON Schema を生成します。

```rust
use rust_config_tree::write_config_schemas;

write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Mark a nested field with `#[schemars(extend("x-tree-split" = true))]` when it
should be generated as its own `*.yaml` template and
`<section>.schema.json` schema. Unmarked nested fields stay in the parent
template and parent schema.

`#[schemars(extend("x-env-only" = true))]` を leaf field に付けると、その値は環境変数からだけ渡すものとして扱われます。生成される template と JSON Schema は env-only field を省略し、その結果空になった parent object も削除します.

generated schemas は `required` constraint を省略します。IDE は補完を提供
できますが、`log.yaml` のような partial file で missing root field を
報告しません。root schema は root file に属する field だけを補完し、nested
section field は各 section schema が補完します。present field は type、enum、
unknown property などの基本的な editor check を受けられます。生成された
`*.schema.json` は具体的な field value が application として合法かどうかを
判断しません。field value validation は code 側で
`#[config(validate = Self::validate)]` として実装し、`load_config` または
`config-validate` で実行します。

generated TOML / YAML / JSON / JSON5 template から schema を bind する場合:

```rust
use rust_config_tree::write_config_templates_with_schema;

write_config_templates_with_schema::<AppConfig>(
    "config.toml",
    "config.example.toml",
    "schemas/myapp.schema.json",
)?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

TOML / YAML の root template は root schema に bind され、split child section
field を補完しません。split section YAML template は対応する section schema に
bind されます。JSON / JSON5 template は VS Code が認識できる root `$schema`
field を受け取ります。VS Code `json.schemas` は代替の bind 方法として残ります。

output format は output path から推定されます。

- `.yaml` と `.yml` は YAML。
- `.toml` は TOML。
- `.json` と `.json5` は JSON5-compatible template。
- unknown extension または extension なしは YAML。

## Schema Bindings

schema path が `schemas/myapp.schema.json` の場合、generated root template は
次を使います。

```toml
#:schema ./schemas/myapp.schema.json
```

```yaml
# yaml-language-server: $schema=./schemas/myapp.schema.json
```

generated section template は section schema を bind します。

```yaml
# log.yaml
# yaml-language-server: $schema=./schemas/log.schema.json
```

生成された JSON / JSON5 template は VS Code が認識する root `$schema` field を書きます。editor settings は任意です:

```json
{
  "json.schemas": [
    {
      "fileMatch": [
        "/config.json",
        "/config.*.json"
      ],
      "url": "./schemas/myapp.schema.json"
    }
  ]
}
```

## Template Source Selection

template generation は source tree を次の順序で選びます。

1. Existing config path。
2. Existing output template path。
3. 新しい empty template tree として扱う output path。

## Mirrored Include Trees

source file が include を宣言している場合、generated template は output directory
下に include path を mirror します。

```yaml
# config.yaml
include:
  - server.yaml
```

`config.example.yaml` を生成すると次を書きます。

```text
config.example.yaml
server.yaml
```

relative include target は output file の parent directory 下に mirror されます。
absolute include target は absolute のままです。

## Opt-in Section Splitting

source file に include がない場合、crate は nested schema section から include
target を導出できます。`server` section を持つ schema では、empty root template
source から次を生成できます。

```text
config.example.yaml
server.yaml
```

root template は include block を受け取り、`server.yaml` は `server`
section だけを含みます。nested section は、その field も `x-tree-split` を持つ場合だけ recursive splitting されます。
