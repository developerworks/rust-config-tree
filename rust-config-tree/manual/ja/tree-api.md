# Tree API

[English](../en/tree-api.html) | [中文](../zh/tree-api.html) | [日本語](tree-api.html) | [한국어](../ko/tree-api.html) | [Français](../fr/tree-api.html) | [Deutsch](../de/tree-api.html) | [Español](../es/tree-api.html) | [Português](../pt/tree-api.html) | [Svenska](../sv/tree-api.html) | [Suomi](../fi/tree-api.html) | [Nederlands](../nl/tree-api.html)

application が `confique` を使わない場合、または traversal result へ直接
アクセスしたい場合は lower-level tree API を使います。

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

## Traversal Rules

tree loader は次を行います。

- source path を字句的に正規化する。
- empty include path を拒否する。
- relative include を宣言元 file から解決する。
- absolute include path を保持する。
- recursive include cycle を検出する。
- 別の include branch で既に読み込まれた file を skip する。

`ConfigTreeOptions` は sibling include traversal を reverse できます。

```rust
use rust_config_tree::{ConfigTreeOptions, IncludeOrder};

let options = ConfigTreeOptions::default().include_order(IncludeOrder::Reverse);
# let _ = options;
```

## Path Helpers

path helper は lexical only です。symbolic link を解決せず、path の存在も要求
しません。

- `absolutize_lexical(path)`
- `normalize_lexical(path)`
- `resolve_include_path(parent_path, include_path)`

