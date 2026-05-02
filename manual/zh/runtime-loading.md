# 运行时加载

[English](../en/runtime-loading.md) | [中文](runtime-loading.md)

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
6. 从 Figment 提取 `confique` layer。
7. 应用 `confique` 代码默认值。
8. 校验并构造最终 schema。

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
