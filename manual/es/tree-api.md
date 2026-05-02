# API de árbol

[English](../en/tree-api.html) | [中文](../zh/tree-api.html) | [日本語](../ja/tree-api.html) | [한국어](../ko/tree-api.html) | [Français](../fr/tree-api.html) | [Deutsch](../de/tree-api.html) | [Español](tree-api.html) | [Português](../pt/tree-api.html) | [Svenska](../sv/tree-api.html) | [Suomi](../fi/tree-api.html) | [Nederlands](../nl/tree-api.html)

Usa la API de árbol de menor nivel cuando la aplicación no use `confique` o
cuando necesite acceso directo a los resultados del recorrido.

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

## Reglas de recorrido

El cargador de árbol:

- normaliza rutas fuente léxicamente;
- rechaza rutas de include vacías;
- resuelve includes relativos desde el archivo que los declaró;
- conserva rutas de include absolutas;
- detecta ciclos de include recursivos;
- omite archivos ya cargados mediante otra rama de include.

`ConfigTreeOptions` puede invertir el recorrido de includes hermanos:

```rust
use rust_config_tree::{ConfigTreeOptions, IncludeOrder};

let options = ConfigTreeOptions::default().include_order(IncludeOrder::Reverse);
# let _ = options;
```

## Ayudantes de ruta

Los ayudantes de ruta son solo léxicos. No resuelven enlaces simbólicos y no
requieren que las rutas existan:

- `absolutize_lexical(path)`
- `normalize_lexical(path)`
- `resolve_include_path(parent_path, include_path)`
