# 来源追踪

[English](../en/source-tracking.html) | [中文](source-tracking.html) | [日本語](../ja/source-tracking.html) | [한국어](../ko/source-tracking.html) | [Français](../fr/source-tracking.html) | [Deutsch](../de/source-tracking.html) | [Español](../es/source-tracking.html) | [Português](../pt/source-tracking.html) | [Svenska](../sv/source-tracking.html) | [Suomi](../fi/source-tracking.html) | [Nederlands](../nl/source-tracking.html)

使用 `load_config_with_figment` 可以保留运行时加载使用的 Figment(配置合并库)
graph(配置图)：

```rust
use rust_config_tree::load_config_with_figment;

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

返回的 Figment(配置合并库) 值可以查询运行时值来源：

```rust
if let Some(metadata) = figment.find_metadata("database.pool_size") {
    let source = metadata.interpolate(
        &figment::Profile::Default,
        &["database", "pool_size"],
    );

    println!("database.pool_size came from {source}");
}
```

对于 `ConfiqueEnvProvider` 提供的值，interpolation(插值结果) 会返回
schema(结构定义) 中声明的原始环境变量名：

```text
database.pool_size came from APP_DATABASE_POOL_SIZE
```

## TRACE 事件

加载器使用 `tracing::trace!` 输出来源追踪事件。只有 TRACE(追踪级别) 启用时，
加载器才会输出这些事件：

```rust
use rust_config_tree::{load_config_with_figment, trace_config_sources};

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;

// 如果配置加载完成后才初始化 tracing subscriber(追踪订阅器)，
// 可以在安装订阅器后再次输出相同的来源事件。
trace_config_sources::<AppConfig>(&figment);
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

每个事件都会使用 `rust_config_tree::config` target(目标)，并包含以下字段：

- `config_key` 表示用点分隔的配置 key(键)。
- `source` 表示渲染后的来源 metadata(元数据)。

如果某个字段只来自 `confique` 默认值，它就没有 Figment(配置合并库) 运行时
metadata(元数据)，并会显示为 `confique default or unset optional field`。
