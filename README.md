# rust-config-tree

`rust-config-tree` provides small, format-agnostic utilities for configuration files
that include other configuration files.

It handles:

- recursive include traversal
- relative include paths resolved from the file declaring them
- lexical path normalization
- include cycle detection
- deterministic traversal order
- mirrored template target collection

The crate does not parse YAML, TOML, JSON, or any other configuration format by
itself. Callers provide the loader that reads one file and returns its include
paths.

## Example

```rust
use std::{fs, path::Path};

use rust_config_tree::{ConfigSource, load_config_tree};

fn read_config(path: &Path) -> std::io::Result<ConfigSource<String>> {
    let content = fs::read_to_string(path)?;
    let includes = content
        .lines()
        .filter_map(|line| line.strip_prefix("include: "))
        .map(Into::into)
        .collect();

    Ok(ConfigSource::new(content, includes))
}

let tree = load_config_tree("config.yaml", read_config)?;
for node in tree.nodes() {
    println!("{}", node.path().display());
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT license

at your option.
