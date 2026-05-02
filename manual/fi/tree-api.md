# Puu-API

[English](../en/tree-api.html) | [中文](../zh/tree-api.html) | [日本語](../ja/tree-api.html) | [한국어](../ko/tree-api.html) | [Français](../fr/tree-api.html) | [Deutsch](../de/tree-api.html) | [Español](../es/tree-api.html) | [Português](../pt/tree-api.html) | [Svenska](../sv/tree-api.html) | [Suomi](tree-api.html) | [Nederlands](../nl/tree-api.html)

Kayta alemman tason puu-APIa, kun sovellus ei kayta `confique`-kirjastoa tai kun se tarvitsee suoran paasan lapikaynnin tuloksiin.

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

## Lapikayntisaannot

Puulataaja:

- normalisoi lahdepolut leksikaalisesti;
- hylkaa tyhjat include-polut;
- ratkaisee suhteelliset includet ne maaritelleesta tiedostosta;
- sailyttaa absoluuttiset include-polut;
- tunnistaa rekursiiviset include-syklit;
- ohittaa tiedostot, jotka on jo ladattu toisen include-haaran kautta.

`ConfigTreeOptions` voi kaantaa saman tason includejen lapikayntijarjestyksen:

```rust
use rust_config_tree::{ConfigTreeOptions, IncludeOrder};

let options = ConfigTreeOptions::default().include_order(IncludeOrder::Reverse);
# let _ = options;
```

## Polkuapurit

Polkuapurit ovat vain leksikaalisia. Ne eivat ratkaise symbolisia linkkeja eivatka vaadi polkujen olemassaoloa:

- `absolutize_lexical(path)`
- `normalize_lexical(path)`
- `resolve_include_path(parent_path, include_path)`
