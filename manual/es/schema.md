# Esquema de configuración

[English](../en/schema.html) | [中文](../zh/schema.html) | [日本語](../ja/schema.html) | [한국어](../ko/schema.html) | [Français](../fr/schema.html) | [Deutsch](../de/schema.html) | [Español](schema.html) | [Português](../pt/schema.html) | [Svenska](../sv/schema.html) | [Suomi](../fi/schema.html) | [Nederlands](../nl/schema.html)

Los esquemas de aplicación son tipos normales de configuración `confique`. El
esquema raíz debe implementar `ConfigSchema` para que `rust-config-tree` pueda
descubrir includes recursivos desde la capa intermedia de `confique`.

```rust
use std::path::PathBuf;

use confique::Config;
use schemars::JsonSchema;
use rust_config_tree::ConfigSchema;

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    database: DatabaseConfig,
}

#[derive(Debug, Config, JsonSchema)]
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

Usa `#[config(nested)]` para secciones estructuradas. Las secciones anidadas
siempre se usan para la carga en tiempo de ejecución. Agrega
`#[schemars(extend("x-tree-split" = true))]` cuando un campo anidado tambien
deba generarse como su propio template `config/*.yaml` y schema
`schemas/*.schema.json`:

```rust
#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    server: ServerConfig,
}
```

La forma YAML natural es:

```yaml
server:
  bind: 127.0.0.1
  port: 8080
```

## Campos solo de entorno

Marca un campo hoja con `#[schemars(extend("x-env-only" = true))]` cuando su valor debe venir solo de una variable de entorno y no debe aparecer en archivos de configuración generados. Las plantillas YAML y los JSON Schemas generados omiten los campos env-only, y también se eliminan los objetos padre que queden vacíos.

```rust
#[config(env = "APP_SECRET")]
#[schemars(extend("x-env-only" = true))]
secret: String,
```

## Validación de valores de campo

Los archivos `*.schema.json` generados sirven solo para completado de IDE y
comprobaciones básicas del editor. No deciden si un valor concreto de campo es
válido para la aplicación.

La validación de valores debe implementarse en código con
`#[config(validate = Self::validate)]`. El validador se ejecuta cuando la
configuración final se carga con `load_config` o se comprueba con
`config-validate`.

## Overrides de sección de plantilla

Cuando una fuente de plantilla no tiene includes, el crate puede derivar
archivos de plantilla hijos desde secciones anidadas del esquema marcadas con `x-tree-split`. La ruta de
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
