# API d'arbre

[English](../en/tree-api.html) | [中文](../zh/tree-api.html) | [日本語](../ja/tree-api.html) | [한국어](../ko/tree-api.html) | [Français](tree-api.html) | [Deutsch](../de/tree-api.html) | [Español](../es/tree-api.html) | [Português](../pt/tree-api.html) | [Svenska](../sv/tree-api.html) | [Suomi](../fi/tree-api.html) | [Nederlands](../nl/tree-api.html)

Utilisez l'API d'arbre de plus bas niveau lorsque l'application n'utilise pas
`confique`, ou lorsqu'elle a besoin d'un acces direct aux resultats de
traversee.

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

## Regles de traversee

Le chargeur d'arbre :

- normalise lexicalement les chemins sources ;
- rejette les chemins d'inclusion vides ;
- resout les inclusions relatives depuis le fichier qui les a declarees ;
- preserve les chemins d'inclusion absolus ;
- detecte les cycles d'inclusion recursifs ;
- ignore les fichiers deja charges par une autre branche d'inclusion.

`ConfigTreeOptions` peut inverser la traversee des inclusions soeurs :

```rust
use rust_config_tree::{ConfigTreeOptions, IncludeOrder};

let options = ConfigTreeOptions::default().include_order(IncludeOrder::Reverse);
# let _ = options;
```

## Assistants de chemin

Les assistants de chemin sont uniquement lexicaux. Ils ne resolvent pas les
liens symboliques et n'exigent pas que les chemins existent :

- `absolutize_lexical(path)`
- `normalize_lexical(path)`
- `resolve_include_path(parent_path, include_path)`

