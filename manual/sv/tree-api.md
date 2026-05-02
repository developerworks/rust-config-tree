# Trad-API

[English](../en/tree-api.html) | [中文](../zh/tree-api.html) | [日本語](../ja/tree-api.html) | [한국어](../ko/tree-api.html) | [Français](../fr/tree-api.html) | [Deutsch](../de/tree-api.html) | [Español](../es/tree-api.html) | [Português](../pt/tree-api.html) | [Svenska](tree-api.html) | [Suomi](../fi/tree-api.html) | [Nederlands](../nl/tree-api.html)

Anvand det lagre niva trad-API:t nar programmet inte anvander `confique`, eller
nar det behover direkt atkomst till traverseringsresultat.

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

## Traverseringsregler

Tradladdaren:

- normaliserar kallsokvagar lexikalt;
- avvisar tomma include-sokvagar;
- loser relativa includes fran filen som deklarerade dem;
- bevarar absoluta include-sokvagar;
- detekterar rekursiva include-cykler;
- hoppar over filer som redan laddats via en annan include-gren.

`ConfigTreeOptions` kan vanda syskonens include-traversering:

```rust
use rust_config_tree::{ConfigTreeOptions, IncludeOrder};

let options = ConfigTreeOptions::default().include_order(IncludeOrder::Reverse);
# let _ = options;
```

## Sokvagshjalpare

Sokvagshjalparna ar bara lexikala. De loser inte symboliska lankar och kraver
inte att sokvagar finns:

- `absolutize_lexical(path)`
- `normalize_lexical(path)`
- `resolve_include_path(parent_path, include_path)`
