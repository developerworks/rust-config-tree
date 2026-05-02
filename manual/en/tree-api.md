# Tree API

[English](tree-api.html) | [中文](../zh/tree-api.html)

Use the lower-level tree API when the application does not use `confique`, or
when it needs direct access to traversal results.

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

The tree loader:

- normalizes source paths lexically;
- rejects empty include paths;
- resolves relative includes from the file that declared them;
- preserves absolute include paths;
- detects recursive include cycles;
- skips files already loaded through another include branch.

`ConfigTreeOptions` can reverse sibling include traversal:

```rust
use rust_config_tree::{ConfigTreeOptions, IncludeOrder};

let options = ConfigTreeOptions::default().include_order(IncludeOrder::Reverse);
# let _ = options;
```

## Path Helpers

The path helpers are lexical only. They do not resolve symbolic links and do not
require paths to exist:

- `absolutize_lexical(path)`
- `normalize_lexical(path)`
- `resolve_include_path(parent_path, include_path)`
