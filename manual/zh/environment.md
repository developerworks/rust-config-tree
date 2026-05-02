# 环境变量

[English](../en/environment.md) | [中文](environment.md)

环境变量名在 schema 中通过 `confique` 声明：

```rust
#[derive(Debug, Config)]
struct DatabaseConfig {
    #[config(env = "APP_DATABASE_URL")]
    url: String,

    #[config(default = 16)]
    #[config(env = "APP_DATABASE_POOL_SIZE")]
    pool_size: u32,
}
```

`rust-config-tree` 从 `confique::Config::META` 读取这些名称，并构建 Figment
provider，将每个环境变量映射到精确字段路径。

不要在这个 crate 的 schema 中使用基于分隔符的 Figment 环境变量映射：

```rust
// Do not use this pattern for rust-config-tree schemas.
Env::prefixed("APP_").split("_")
Env::prefixed("APP_").split("__")
```

`split("_")` 会把下划线当成嵌套 key 分隔符。这样
`APP_DATABASE_POOL_SIZE` 会变成类似 `database.pool.size` 的路径，与
`pool_size` 这种 Rust 字段名冲突。

使用 `ConfiqueEnvProvider` 时，映射是显式的：

```text
APP_DATABASE_POOL_SIZE -> database.pool_size
```

单个下划线仍然只是环境变量名的一部分。Figment 不会猜测嵌套规则。

## Dotenv 加载

在运行时 provider 求值之前，加载器会从 root config 文件所在目录开始向上
查找 `.env` 文件。

已有的进程环境变量会被保留。`.env` 中的值只填充缺失的环境变量。

示例：

```dotenv
APP_SERVER_PORT=9000
APP_DATABASE_POOL_SIZE=64
```

当 schema 声明了匹配的 `#[config(env = "...")]` 属性时，这些变量会覆盖
配置文件中的值。

## 值解析

桥接 provider 让 Figment 解析环境变量值。它不会调用 `confique` 的
`parse_env` hook。复杂值优先放在配置文件中，除非 Figment 的环境变量值
语法非常适合目标类型。
