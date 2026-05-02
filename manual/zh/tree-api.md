# Tree API

[English](../en/tree-api.md) | [中文](tree-api.md)

应用不使用 `confique`，或者需要直接访问遍历结果时，可以使用低层 tree API。

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

tree loader 会：

- 对 source path 做词法归一化；
- 拒绝空 include path；
- 从声明文件解析相对 include；
- 保留绝对 include path；
- 检测递归 include 循环；
- 跳过已经从其他 include 分支加载过的文件。

`ConfigTreeOptions` 可以反转同级 include 遍历顺序：

```rust
use rust_config_tree::{ConfigTreeOptions, IncludeOrder};

let options = ConfigTreeOptions::default().include_order(IncludeOrder::Reverse);
# let _ = options;
```

## 路径辅助函数

路径辅助函数只做词法处理。它们不解析符号链接，也不要求路径存在：

- `absolutize_lexical(path)`
- `normalize_lexical(path)`
- `resolve_include_path(parent_path, include_path)`
