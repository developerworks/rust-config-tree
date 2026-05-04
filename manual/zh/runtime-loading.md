# 运行时加载

[English](../en/runtime-loading.html) | [中文](runtime-loading.html) | [日本語](../ja/runtime-loading.html) | [한국어](../ko/runtime-loading.html) | [Français](../fr/runtime-loading.html) | [Deutsch](../de/runtime-loading.html) | [Español](../es/runtime-loading.html) | [Português](../pt/runtime-loading.html) | [Svenska](../sv/runtime-loading.html) | [Suomi](../fi/runtime-loading.html) | [Nederlands](../nl/runtime-loading.html)

运行时加载会把职责分别交给 Figment(配置合并库) 和 confique(配置结构定义库)：

```text
Figment(配置合并库):
  运行时文件加载
  运行时环境变量加载
  运行时来源元数据

confique(配置结构定义库):
  结构定义元数据
  默认值
  校验
  配置模板
```

主要 API(应用程序接口) 如下：

```rust
use rust_config_tree::load_config;

let config = load_config::<AppConfig>("config.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

当应用需要来源 metadata(元数据) 时，可以使用 `load_config_with_figment`：

```rust
use rust_config_tree::load_config_with_figment;

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;
# let _ = (config, figment);
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

## 加载步骤

高层加载器会按以下步骤执行：

1. 加载器会对 root config(根配置) 路径做词法解析。
2. 加载器会从 root config(根配置) 所在目录开始向上查找，并加载第一个 `.env` 文件。
3. 加载器会将每个配置文件加载成部分 layer(层)，用于发现 include(包含文件)。
4. 加载器会从发现的配置文件构建 Figment(配置合并库) graph(配置图)。
5. 加载器会以高于文件的优先级合并 `ConfiqueEnvProvider`。
6. 应用可以选择合并自己的 CLI override(命令行覆盖值)。
7. 加载器会从 Figment(配置合并库) 提取 `confique` layer(层)。
8. 应用 `confique` 代码默认值。
9. 加载器会校验并构造最终 schema(结构定义)。

`load_config` 和 `load_config_with_figment` 执行第 1-5 步和第 7-9 步。
第 6 步属于应用语义，因为这个 crate(软件包) 无法推断某个
CLI flag(命令行参数) 应该映射到哪个 schema(结构定义) 字段。

## 文件格式

运行时文件 provider(值提供器) 会根据配置路径的扩展名选择文件格式：

- `.yaml` 和 `.yml` 使用 YAML。
- `.toml` 使用 TOML。
- `.json` 和 `.json5` 使用 JSON。
- 未知或缺失扩展名使用 YAML。

模板生成仍使用 confique(配置结构定义库) 的 YAML、TOML 和
JSON5-compatible(JSON5 兼容) 模板渲染器。

## Include(包含) 优先级

高层加载器合并文件 provider(值提供器) 时，被 include(包含) 的文件优先级低于
include(包含) 它的文件。root config(根配置) 文件拥有最高的文件优先级。

环境变量优先级高于所有配置文件。`confique` 默认值只用于运行时
provider(值提供器) 没有提供的值。

当 CLI override(命令行覆盖值) 在 `build_config_figment` 之后合并时，完整优先级如下：

```text
命令行覆盖值
  > 环境变量
    > 配置文件
      > confique 代码默认值
```

命令行语法不是由 `rust-config-tree` 定义的。只要应用把 `--server-port`
解析出的值映射进嵌套 serialized provider(序列化值提供器)，这个值就可以覆盖
`server.port`。
`--server.port` 或 `a.b.c` 这种点分路径语法只有在应用自己实现时才存在。

因此 CLI(命令行接口) 优先级只作用于应用 override provider(覆盖值提供器)
中存在的 key(键)。它适合临时、频繁调整的运行参数。长期稳定配置应留在
配置文件里。

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
