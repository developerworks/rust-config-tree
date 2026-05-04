# 配置结构

[English](../en/schema.html) | [中文](schema.html) | [日本語](../ja/schema.html) | [한국어](../ko/schema.html) | [Français](../fr/schema.html) | [Deutsch](../de/schema.html) | [Español](../es/schema.html) | [Português](../pt/schema.html) | [Svenska](../sv/schema.html) | [Suomi](../fi/schema.html) | [Nederlands](../nl/schema.html)

应用的 schema(结构定义) 是普通的 `confique` config(配置) 类型。
root schema(根结构定义) 必须实现 `ConfigSchema`，这样 `rust-config-tree`
才能从中间 `confique` layer(层) 中发现递归 include(包含文件)。

```rust
use std::path::PathBuf;

use confique::Config;
use schemars::JsonSchema;
use rust_config_tree::ConfigSchema;

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    database: DatabaseConfig,
}

#[derive(Debug, Config, JsonSchema)]
struct DatabaseConfig {
    #[config(env = "APP_DATABASE_URL")]
    url: String,

    #[config(default = 16)]
    #[config(env = "APP_DATABASE_POOL_SIZE")]
    pool_size: u32,
}

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}
```

## Include(包含) 字段

include(包含) 字段可以使用任意名称。`rust-config-tree` 只通过
`ConfigSchema::include_paths` 读取这个字段。

这个字段通常应有空默认值：

```rust
#[config(default = [])]
include: Vec<PathBuf>,
```

加载器会接收每个文件的部分 layer(层)。这样加载器可以在最终
schema(结构定义) 合并和校验之前发现子配置文件。

## 嵌套 Section(配置段)

使用 `#[config(nested)]` 表示结构化 section(配置段)。嵌套 section(配置段)
一定会影响运行时加载。如果某个 nested(嵌套) 字段还需要生成独立的
`config/*.yaml` 模板和 `schemas/*.schema.json` schema(结构定义)，就给这个字段加上
`#[schemars(extend("x-tree-split" = true))]`：

```rust
#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    server: ServerConfig,
}
```

对应的自然 YAML 形状为：

```yaml
server:
  bind: 127.0.0.1
  port: 8080
```

## 环境变量专用字段

当 leaf(叶子) 字段只能由环境变量提供，并且不应该出现在生成的配置文件中时，
可以使用 `#[schemars(extend("x-env-only" = true))]`。生成的 YAML 模板和
JSON Schema(JSON 结构定义) 会省略 env-only(仅环境变量) 字段；如果父对象因此变空，
生成器也会删除这个父对象。

```rust
#[config(env = "APP_SECRET")]
#[schemars(extend("x-env-only" = true))]
secret: String,
```

## 字段值合法性校验

生成的 `*.schema.json` 文件只用于 IDE(集成开发环境) 补全和基础编辑期检查，
不负责判断具体字段值对应用是否合法。

字段值合法性应在代码中通过 `#[config(validate = Self::validate)]` 实现。
当 `load_config` 加载最终配置，或者 `config-validate` 检查最终配置时，
运行时会执行这个校验。

## 模板 Section(配置段) 路径覆盖

当模板 source(来源) 没有 include(包含文件) 时，crate(软件包) 可以从带
`x-tree-split` 标记的嵌套 schema section(结构定义配置段) 推导子模板文件。
默认顶层路径是 `config/<section>.yaml`。

使用 `template_path_for_section` 覆盖路径：

```rust
impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }

    fn template_path_for_section(section_path: &[&str]) -> Option<PathBuf> {
        match section_path {
            ["database"] => Some(PathBuf::from("examples/database.yaml")),
            _ => None,
        }
    }
}
```
