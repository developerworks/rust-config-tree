# Tree API(树形接口)

[English](../en/tree-api.html) | [中文](tree-api.html) | [日本語](../ja/tree-api.html) | [한국어](../ko/tree-api.html) | [Français](../fr/tree-api.html) | [Deutsch](../de/tree-api.html) | [Español](../es/tree-api.html) | [Português](../pt/tree-api.html) | [Svenska](../sv/tree-api.html) | [Suomi](../fi/tree-api.html) | [Nederlands](../nl/tree-api.html)

当应用不使用 `confique`，或者需要直接访问遍历结果时，应用可以使用低层
tree API(树形接口)。

```rust
use std::{
    fs,
    io,
    path::{Path, PathBuf},
};

use rust_config_tree::{ConfigSource, load_config_tree};

fn load_source(path: &Path) -> io::Result<ConfigSource<String>> {
    let content = fs::read_to_string(path)?;
    let includes = content
        .lines()
        .filter_map(|line| line.strip_prefix("include: "))
        .map(PathBuf::from)
        .collect();

    Ok(ConfigSource::new(content, includes))
}

let tree = load_config_tree("config.yaml", load_source)?;

for node in tree.nodes() {
    println!("{}", node.path().display());
}
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

## 遍历规则

tree loader(树形加载器) 会执行以下操作：

- 它会对 source path(来源路径) 做词法归一化。
- 它会拒绝空 include path(包含路径)。
- 它会从声明文件解析相对 include(包含)。
- 它会保留绝对 include path(包含路径)。
- 它会检测递归 include(包含) 循环。
- 它会跳过已经从其他 include(包含) 分支加载过的文件。

`ConfigTreeOptions` 可以反转同级 include(包含) 的遍历顺序：

```rust
use rust_config_tree::{ConfigTreeOptions, IncludeOrder};

let options = ConfigTreeOptions::default().include_order(IncludeOrder::Reverse);
# let _ = options;
```

## 路径辅助函数

路径辅助函数只做词法处理。它们不解析符号链接，也不要求路径存在：

- `absolutize_lexical(path)` 会把路径转换成词法绝对路径。
- `normalize_lexical(path)` 会对路径做词法归一化。
- `resolve_include_path(parent_path, include_path)` 会根据父路径解析 include(包含)
  路径。
