# 透明数组 Section(配置段)

[English](../en/transparent-sections.html) | [中文](transparent-sections.html)

## 概述

**透明数组 Section**(transparent array section) 让 split(拆分) 配置段在 YAML(数据序列化格式) 里表现为数组, 在 Rust 里通过内部 `items` 字段存储. 加载器, 模板生成器和 section schema(配置段结构定义) 都会把该段当作数组处理, 而不是 `{ items: [...] }` 对象.

一句话: 单文件里写 `children: [...]`, split 文件里只写 `[...]`, 不需要 `items:` 包裹.

适用场景: 列表型配置(例如 worker 声明, 路由表, 插件列表)需要独立 split 文件, 又希望 split 文件保持简洁的数组体.

## Schema(结构定义) 标记

透明数组 Section 需要同时标记 `x-tree-split` 和 `x-tree-transparent-array`:

```rust
use rust_config_tree::transparent_array_section;

transparent_array_section! {
    /// Child declarations stored as a transparent array section.
    pub struct ChildrenSection {
        #[config(default = [{ "name": "worker" }])]
        pub items: Vec<ChildDeclaration>,
    }
}

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(nested)]
    #[schemars(extend(
        "x-tree-split" = true,
        "x-tree-transparent-array" = true
    ))]
    children: ChildrenSection,
}
```

| 扩展                       | 作用                                                     |
| -------------------------- | -------------------------------------------------------- |
| `x-tree-split`             | 生成独立 `children.yaml` 模板和 `children.schema.json`   |
| `x-tree-transparent-array` | 运行时与模板层把该段当作 YAML 数组, 而不是嵌套对象       |
| `x-tree-inner-field`(可选) | 覆盖 confique(配置结构定义库) 内部字段名, 默认 `"items"` |

## Rust 类型选择

### 宏 `transparent_array_section!`

宏会为 Section 生成 `len`, `is_empty`, `as_slice`, `into_vec`, 以及 `Deref`/`DerefMut` 到 `Vec<T>`. 每个 Section 可以有自己的 `#[config(default = ...)]` 模板样例.

### 泛型 `ArraySection<T>`

若不需要宏, 可以直接使用 `ArraySection<T>`:

```rust
use rust_config_tree::ArraySection;

#[derive(Debug, Config, JsonSchema)]
struct ChildrenSection {
    #[config(default = [{ "name": "worker" }])]
    items: Vec<ChildDeclaration>,
}
```

`ArraySection<T>` 上只能有一个 `#[config(default)]`. 若多个 Section 需要不同的模板 default(默认值), 请为每个 Section 定义独立 struct, 或使用宏.

## YAML 形状

### 单文件

根配置里直接写数组:

```yaml
children:
  - name: api
  - name: worker
```

### Split 文件

根配置引用 split 文件:

```yaml
include:
  - children.yaml
mode: demo
```

`children.yaml` **只写数组体**, 不要写 section 根键或 `items:`:

```yaml
- name: api
- name: worker
```

### 加载器兼容的形状

`load_config` 接受以下三种写法:

1. 透明数组: `children: [...]`
2. 显式 inner field(内部字段): `children:\n  items: [...]`
3. body-only split 文件: `children.yaml` 内容为 `[...]`

加载器会把 split 文件 merge(合并) 成 `children: { items: [...] }`, 再交给 confique 反序列化.

## 运行时 default(默认值) 与模板 default

模板生成会使用 `#[config(default = ...)]` 写入样例条目(例如 `worker`).

若运行时配置**完全省略**该 transparent section, 库会通过 `TransparentSectionTracker` 注入 `{ items: [] }`, 避免 confique 模板 default 在运行时 phantom(幽灵) 注入.

因此:

- **模板 default**: 指导用户如何填写配置, 出现在 `generate-template` 输出里.
- **运行时 default**: 省略 section 时得到空数组, 而不是模板里的样例 worker.

## 模板与 Schema(结构定义)

生成 `children.schema.json` 时, 顶层类型为 `array`, IDE(集成开发环境) 在编辑 `children.yaml` 时直接补全数组项.

模板生成输出 block YAML(块状 YAML) 数组体:

- 不写 `children:` 根键
- 不写 `items:` 包裹
- 不使用 flow 风格 `[{ ... }]`

根模板 `config.example.yaml` 会包含 `include: [children.yaml]`.

## 访问数据

```rust
config.children.len();
config.children.is_empty();
config.children.as_slice();
let vec: Vec<ChildDeclaration> = config.children.into_vec();
```

宏或 `ArraySection` 都支持上述 API(应用程序接口).

## 完整示例

仓库提供可运行示例:

```bash
cargo run --example transparent_array_section
```

该示例会:

1. 写入 split 配置到临时目录
2. 生成 section schema(配置段结构定义)
3. 调用 `load_config` 并验证透明数组加载

下游项目参考: [rust-supervisor split-config](https://github.com/developerworks/rust-supervisor/blob/main/manual/zh/split-config.md) 展示了 `groups` 与 `children` 的实际用法.

## 相关页面

- [配置结构](schema.md) — nested section(嵌套配置段) 与 split 标记
- [模板生成](templates.md) — split section 模板输出规则
- [运行时加载](runtime-loading.md) — 加载步骤与 merge 优先级
- [IDE(集成开发环境) 补全](ide-completions.md) — section schema 绑定
