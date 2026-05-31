# Tree-API

[English](../en/tree-api.html) | [中文](../zh/tree-api.html) | [日本語](../ja/tree-api.html) | [한국어](../ko/tree-api.html) | [Français](../fr/tree-api.html) | [Deutsch](tree-api.html) | [Español](../es/tree-api.html) | [Português](../pt/tree-api.html) | [Svenska](../sv/tree-api.html) | [Suomi](../fi/tree-api.html) | [Nederlands](../nl/tree-api.html)

Verwende die untergeordnete Tree-API, wenn die Anwendung `confique` nicht
verwendet oder direkten Zugriff auf Traversierungsergebnisse braucht.

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

## Traversierungsregeln

Der Tree-Loader:

- normalisiert Quellpfade lexikalisch;
- weist leere Include-Pfade zurueck;
- loest relative Includes von der Datei auf, die sie deklariert hat;
- behaelt absolute Include-Pfade bei;
- erkennt rekursive Include-Zyklen;
- ueberspringt Dateien, die bereits ueber einen anderen Include-Zweig geladen
  wurden.

`ConfigTreeOptions` kann die Traversierung von Geschwister-Includes umkehren:

```rust
use rust_config_tree::{ConfigTreeOptions, IncludeOrder};

let options = ConfigTreeOptions::default().include_order(IncludeOrder::Reverse);
# let _ = options;
```

## Pfadhelfer

Die Pfadhelfer arbeiten rein lexikalisch. Sie loesen keine symbolischen Links
auf und verlangen nicht, dass Pfade existieren:

- `absolutize_lexical(path)`
- `normalize_lexical(path)`
- `resolve_include_path(parent_path, include_path)`
