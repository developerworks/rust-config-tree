# API de arvore

[English](../en/tree-api.html) | [中文](../zh/tree-api.html) | [日本語](../ja/tree-api.html) | [한국어](../ko/tree-api.html) | [Français](../fr/tree-api.html) | [Deutsch](../de/tree-api.html) | [Español](../es/tree-api.html) | [Português](tree-api.html) | [Svenska](../sv/tree-api.html) | [Suomi](../fi/tree-api.html) | [Nederlands](../nl/tree-api.html)

Use a API de arvore de nivel mais baixo quando a aplicacao nao usa `confique`
ou quando precisa de acesso direto aos resultados da travessia.

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

## Regras de travessia

O carregador de arvore:

- normaliza caminhos de origem lexicalmente;
- rejeita caminhos de include vazios;
- resolve includes relativos a partir do arquivo que os declarou;
- preserva caminhos de include absolutos;
- detecta ciclos de include recursivos;
- ignora arquivos ja carregados por outro ramo de include.

`ConfigTreeOptions` pode inverter a travessia de includes irmaos:

```rust
use rust_config_tree::{ConfigTreeOptions, IncludeOrder};

let options = ConfigTreeOptions::default().include_order(IncludeOrder::Reverse);
# let _ = options;
```

## Auxiliares de caminho

Os auxiliares de caminho sao apenas lexicais. Eles nao resolvem links simbolicos
e nao exigem que caminhos existam:

- `absolutize_lexical(path)`
- `normalize_lexical(path)`
- `resolve_include_path(parent_path, include_path)`

