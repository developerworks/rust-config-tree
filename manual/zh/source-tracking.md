# 来源追踪

[English](../en/source-tracking.md) | [中文](source-tracking.md)

使用 `load_config_with_figment` 保留运行时加载使用的 Figment graph：

```rust
use rust_config_tree::load_config_with_figment;

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

返回的 Figment 值可以查询运行时值来源：

```rust
if let Some(metadata) = figment.find_metadata("database.pool_size") {
    let source = metadata.interpolate(
        &figment::Profile::Default,
        &["database", "pool_size"],
    );

    println!("database.pool_size came from {source}");
}
```

对于 `ConfiqueEnvProvider` 提供的值，interpolation 返回 schema 中声明的
原始环境变量名：

```text
database.pool_size came from APP_DATABASE_POOL_SIZE
```

## TRACE 事件

加载器使用 `tracing::trace!` 输出来源追踪事件。只有 TRACE 启用时才会输出：

```rust
use rust_config_tree::{load_config_with_figment, trace_config_sources};

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;

// If the tracing subscriber is initialized after config loading, emit the
// same source events again after installing the subscriber.
trace_config_sources::<AppConfig>(&figment);
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

每个事件使用 `rust_config_tree::config` target，并包含：

- `config_key`：点分隔的配置 key。
- `source`：渲染后的来源 metadata。

只来自 `confique` 默认值的字段没有 Figment 运行时 metadata，会显示为
`confique default or unset optional field`。
