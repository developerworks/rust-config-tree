# Tree API

[English](../en/tree-api.html) | [中文](../zh/tree-api.html) | [日本語](../ja/tree-api.html) | [한국어](../ko/tree-api.html) | [Français](../fr/tree-api.html) | [Deutsch](../de/tree-api.html) | [Español](../es/tree-api.html) | [Português](../pt/tree-api.html) | [Svenska](../sv/tree-api.html) | [Suomi](../fi/tree-api.html) | [Nederlands](tree-api.html)

Gebruik de lagere tree-API wanneer de toepassing geen `confique` gebruikt, of
wanneer directe toegang tot traversalresultaten nodig is.

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

## Traversalregels

De tree-loader:

- normaliseert bronpaden lexicaal;
- weigert lege include-paden;
- lost relatieve includes op vanuit het bestand dat ze declareerde;
- behoudt absolute include-paden;
- detecteert recursieve include-cycli;
- slaat bestanden over die al via een andere include-tak zijn geladen.

`ConfigTreeOptions` kan sibling-include-traversal omkeren:

```rust
use rust_config_tree::{ConfigTreeOptions, IncludeOrder};

let options = ConfigTreeOptions::default().include_order(IncludeOrder::Reverse);
# let _ = options;
```

## Padhulpen

De padhulpen zijn alleen lexicaal. Ze lossen geen symbolische links op en
vereisen niet dat paden bestaan:

- `absolutize_lexical(path)`
- `normalize_lexical(path)`
- `resolve_include_path(parent_path, include_path)`
