# Esquema de configuración

[English](../en/schema.html) | [中文](../zh/schema.html) | [日本語](../ja/schema.html) | [한국어](../ko/schema.html) | [Français](../fr/schema.html) | [Deutsch](../de/schema.html) | [Español](schema.html) | [Português](../pt/schema.html) | [Svenska](../sv/schema.html) | [Suomi](../fi/schema.html) | [Nederlands](../nl/schema.html)

Los esquemas de aplicación son tipos normales de configuración `confique`. El
esquema raíz debe implementar `ConfigSchema` para que `rust-config-tree` pueda
descubrir includes recursivos desde la capa intermedia de `confique`.

```rust
use std::path::PathBuf;

use confique::Config;
use rust_config_tree::ConfigSchema;

#[derive(Debug, Config)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(nested)]
    database: DatabaseConfig,
}

#[derive(Debug, Config)]
struct DatabaseConfig {
    #[config(env = "APP_DATABASE_URL")]
    url: String,

    #[config(default = 16)]
    #[config(env = "APP_DATABASE_POOL_SIZE")]
    pool_size: u32,
}

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}
```

## Campo include

El campo include puede tener cualquier nombre. `rust-config-tree` solo lo
conoce mediante `ConfigSchema::include_paths`.

Normalmente el campo debería tener un valor por defecto vacío:

```rust
#[config(default = [])]
include: Vec<PathBuf>,
```

El cargador recibe una capa parcialmente cargada para cada archivo. Eso le
permite descubrir archivos de configuración hijos antes de fusionar y validar
el esquema final.

## Secciones anidadas

Usa `#[config(nested)]` para secciones estructuradas. Las secciones anidadas son
importantes tanto para la carga en tiempo de ejecución como para la división de
plantillas:

```rust
#[derive(Debug, Config)]
struct AppConfig {
    #[config(nested)]
    server: ServerConfig,
}
```

La forma YAML natural es:

```yaml
server:
  bind: 127.0.0.1
  port: 8080
```

## Overrides de sección de plantilla

Cuando una fuente de plantilla no tiene includes, el crate puede derivar
archivos de plantilla hijos desde secciones anidadas del esquema. La ruta de
primer nivel por defecto es `config/<section>.yaml`.

Sobrescribe esa ruta con `template_path_for_section`:

```rust
impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }

    fn template_path_for_section(section_path: &[&str]) -> Option<PathBuf> {
        match section_path {
            ["database"] => Some(PathBuf::from("examples/database.yaml")),
            _ => None,
        }
    }
}
```
