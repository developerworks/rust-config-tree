# 运行时加载

[English](../en/runtime-loading.html) | [中文](runtime-loading.html)

运行时加载刻意拆分给 Figment 和 confique：

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

主要 API：

```rust
use rust_config_tree::load_config;

let config = load_config::<AppConfig>("config.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

应用需要来源 metadata 时，使用 `load_config_with_figment`：

```rust
use rust_config_tree::load_config_with_figment;

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;
# let _ = (config, figment);
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

## 加载步骤

高层加载器执行这些步骤：

1. 对 root config 路径做词法解析。
2. 从 root config 所在目录开始向上查找并加载第一个 `.env` 文件。
3. 将每个配置文件加载成部分 layer，用于发现 include。
4. 从发现的配置文件构建 Figment graph。
5. 以高于文件的优先级合并 `ConfiqueEnvProvider`。
6. 可选地合并应用自己的 CLI override。
7. 从 Figment 提取 `confique` layer。
8. 应用 `confique` 代码默认值。
9. 校验并构造最终 schema。

`load_config` 和 `load_config_with_figment` 执行第 1-5 步和第 7-9 步。
第 6 步属于应用语义，因为这个 crate 无法推断某个 CLI flag 应该映射到
哪个 schema 字段。

## 文件格式

运行时文件 provider 由配置路径扩展名选择：

- `.yaml` 和 `.yml` 使用 YAML。
- `.toml` 使用 TOML。
- `.json` 和 `.json5` 使用 JSON。
- 未知或缺失扩展名使用 YAML。

模板生成仍使用 confique 的 YAML、TOML 和 JSON5-compatible 模板渲染器。

## Include 优先级

高层加载器合并文件 provider 时，被 include 的文件优先级低于 include 它的
文件。root config 文件拥有最高文件优先级。

环境变量优先级高于所有配置文件。`confique` 默认值只用于运行时 provider
没有提供的值。

当 CLI override 在 `build_config_figment` 之后合并时，完整优先级为：

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```

命令行语法不是由 `rust-config-tree` 定义的。只要应用把 `--server-port`
解析出的值映射进嵌套 serialized provider，它就可以覆盖 `server.port`。
`--server.port` 或 `a.b.c` 这种点分路径语法只有在应用自己实现时才存在。

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
