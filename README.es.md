# rust-config-tree

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

`rust-config-tree` proporciona carga de árboles de configuración y ayudantes de
CLI para aplicaciones Rust que usan archivos de configuración por capas.

Manual del proyecto: <https://developerworks.github.io/rust-config-tree/>.
Los manuales por idioma se publican como sitios mdBook independientes con
enlaces para cambiar de idioma.

Gestiona:

- cargar un esquema `confique` en un objeto de configuración directamente
  utilizable mediante proveedores de Figment en tiempo de ejecución
- manejadores de comandos `config-template`, `completions` e
  `install-completions`
- generación de JSON Schema Draft 7 para la raíz y las secciones, útil para
  completado y validación en editores
- generación de plantillas de configuración para YAML, TOML, JSON y JSON5
- directivas de esquema para plantillas TOML y YAML sin añadir campos en tiempo
  de ejecución
- recorrido recursivo de includes
- carga de `.env` antes de fusionar valores de entorno
- seguimiento de origen mediante metadatos de Figment
- logs de seguimiento de origen en nivel TRACE mediante `tracing`
- rutas de include relativas resueltas desde el archivo que las declara
- normalización léxica de rutas
- detección de ciclos de include
- orden de recorrido determinista
- recopilación reflejada de destinos de plantilla
- división opt-in de plantillas YAML para secciones marcadas con `x-tree-split`

Las aplicaciones proporcionan su esquema derivando `confique::Config` e
implementando `ConfigSchema` para exponer el campo de includes del esquema.

## Instalación

```toml
[dependencies]
rust-config-tree = "0.1"
confique = { version = "0.4", features = ["yaml", "toml", "json5"] }
figment = { version = "0.10", features = ["yaml", "env"] }
schemars = { version = "1", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
clap = { version = "4", features = ["derive"] }
```

## Esquema de configuración

El esquema de la aplicación es dueño del campo de includes. `rust-config-tree`
solo necesita un pequeño adaptador que extrae los includes de la capa
intermedia de `confique`.

```rust
use std::path::PathBuf;

use confique::Config;
use schemars::JsonSchema;
use rust_config_tree::ConfigSchema;

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(default = "paper")]
    mode: String,

    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    server: ServerConfig,
}

#[derive(Debug, Config, JsonSchema)]
struct ServerConfig {
    #[config(default = 8080)]
    port: u16,
}

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}
```

Las rutas de include relativas se resuelven desde el archivo que las declara:

```yaml
# config.yaml
include:
  - config/server.yaml

mode: shadow
```

```yaml
# config/server.yaml
server:
  port: 7777
```

Carga el esquema final con `load_config`:

```rust
use rust_config_tree::load_config;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = load_config::<AppConfig>("config.yaml")?;
    println!("{config:#?}");

    Ok(())
}
```

`load_config` carga el primer archivo `.env` encontrado al caminar hacia arriba
desde el directorio del archivo de configuración raíz antes de pedir a Figment
que lea las variables de entorno declaradas por el esquema. Los valores ya
presentes en el entorno del proceso se conservan y tienen prioridad sobre los
valores de `.env`.

La carga de configuración en tiempo de ejecución se realiza mediante Figment.
`confique` sigue siendo responsable de los metadatos del esquema, valores por
defecto, validación y generación de plantillas. Los nombres de variables de
entorno se leen de `#[config(env = "...")]`; el cargador no usa
`Env::split("_")` ni `Env::split("__")`, por lo que una variable como
`APP_DATABASE_POOL_SIZE` puede mapearse a un campo llamado
`database.pool_size`.

`load_config` no lee argumentos de línea de comandos porque las flags de CLI
son específicas de la aplicación. Añade overrides de CLI fusionando un
proveedor después de `build_config_figment` y luego valida con
`load_config_from_figment`:

Los nombres de flags de CLI no se derivan de rutas de configuración. Usa flags
normales de aplicación como `--server-port` o `--database-url`; no dependas de
`--server.port` o `a.b.c` salvo que la aplicación implemente deliberadamente
ese parser. La forma serializada anidada del override decide qué clave de
configuración se sobrescribe.

Solo los valores representados en el proveedor `CliOverrides` de la aplicación
pueden sobrescribir la configuración. Esto está pensado para parámetros de
tiempo de ejecución que se ajustan con frecuencia, donde cambiar una flag para
una ejecución es mejor que editar un archivo de configuración. Mantén la
configuración estable en archivos y expón como flags solo overrides temporales
deliberados.

```rust
use figment::providers::Serialized;
use serde::Serialize;
use rust_config_tree::{build_config_figment, load_config_from_figment};

#[derive(Debug, Serialize)]
struct CliOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<String>,
}

fn load_with_cli_overrides(cli_mode: Option<String>) -> Result<AppConfig, Box<dyn std::error::Error + Send + Sync>> {
    let cli_overrides = CliOverrides {
        mode: cli_mode,
    };

    let figment = build_config_figment::<AppConfig>("config.yaml")?
        .merge(Serialized::defaults(cli_overrides));

    let config = load_config_from_figment::<AppConfig>(&figment)?;
    Ok(config)
}
```

Con overrides de CLI fusionados de esta forma, la precedencia en tiempo de
ejecución es:

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```

Usa `load_config_with_figment` cuando el llamador necesita metadatos de origen:

```rust
use rust_config_tree::load_config_with_figment;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;

    if let Some(metadata) = figment.find_metadata("mode") {
        let source = metadata.interpolate(&figment::Profile::Default, &["mode"]);
        println!("mode came from {source}");
    }

    println!("{config:#?}");

    Ok(())
}
```

El cargador también emite seguimiento de origen de configuración con
`tracing::trace!`. Esos eventos solo se producen cuando TRACE está habilitado
por el subscriber de `tracing` de la aplicación. Si `tracing` se inicializa
después de cargar la configuración, llama a
`trace_config_sources::<AppConfig>(&figment)` después de instalar el subscriber.

## Generación de plantillas

Las plantillas se renderizan con el mismo esquema y las mismas reglas de
recorrido de includes. El formato de salida se infiere de la ruta de salida:

- `.yaml` y `.yml` generan YAML
- `.toml` genera TOML
- `.json` y `.json5` generan plantillas compatibles con JSON5
- extensiones desconocidas o ausentes generan YAML

Usa `write_config_schemas` para crear JSON Schemas Draft 7 para la
configuración raíz y las secciones marcadas con `x-tree-split`. Los esquemas generados omiten
restricciones `required` para que los IDE puedan ofrecer completado en archivos
de configuración parciales sin informar campos faltantes:

```rust
use rust_config_tree::write_config_schemas;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;

    Ok(())
}
```

Mark a nested field with `#[schemars(extend("x-tree-split" = true))]` when it
should be generated as its own `config/*.yaml` template and
`schemas/*.schema.json` schema. Unmarked nested fields stay in the parent
template and parent schema.

Para un esquema con secciones `server` y `log` marcadas con `x-tree-split`, esto escribe
`schemas/myapp.schema.json`, `schemas/server.schema.json` y
`schemas/log.schema.json`. El esquema raíz contiene solo campos que pertenecen
al archivo de configuración raíz, como `include` y campos escalares raíz. Omite
deliberadamente las propiedades de secciones divididas, de modo que `server` y
`log` solo se completan al editar sus propios archivos YAML de sección.

Usa `write_config_templates` para crear una plantilla raíz y todos los archivos
de plantilla alcanzables desde su árbol de includes:

```rust
use rust_config_tree::write_config_templates;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;

    Ok(())
}
```

Usa `write_config_templates_with_schema` cuando las plantillas TOML y YAML
generadas deban enlazar esos esquemas para completado y validación en el IDE:

```rust
use rust_config_tree::write_config_templates_with_schema;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_templates_with_schema::<AppConfig>(
        "config.toml",
        "config.example.toml",
        "schemas/myapp.schema.json",
    )?;

    Ok(())
}
```

Los destinos TOML/YAML raíz enlazan el esquema raíz y no completan campos de
secciones hijas. Los destinos YAML de secciones divididas enlazan su esquema de
sección correspondiente; por ejemplo, `config/log.yaml` recibe
`# yaml-language-server: $schema=../schemas/log.schema.json`. Los destinos JSON
y JSON5 no reciben deliberadamente un campo `$schema`; enlázalos con ajustes
del editor como `json.schemas` de VS Code.

La generación de plantillas elige su árbol fuente en este orden:

- una ruta de configuración existente
- una ruta de plantilla de salida existente
- la ruta de salida, tratada como un nuevo árbol de plantillas vacío

Si un nodo fuente no tiene lista de includes, `rust-config-tree` deriva
archivos de plantilla hijos desde las secciones anidadas de `confique` marcadas con `x-tree-split`. Con el
esquema anterior, una fuente `config.example.yaml` vacía produce:

```text
config.example.yaml
config/server.yaml
```

La plantilla raíz recibe un bloque include para `config/server.yaml`. Los
destinos YAML que se mapean a una sección anidada, como `config/server.yaml`,
contienen solo esa sección. Las secciones anidadas mas profundas solo se dividen
recursivamente cuando esos campos tambien llevan `x-tree-split`.

Sobrescribe `template_path_for_section` cuando una sección deba generarse en
una ruta distinta:

```rust
use std::path::PathBuf;

use confique::Config;
use rust_config_tree::ConfigSchema;

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }

    fn template_path_for_section(section_path: &[&str]) -> Option<PathBuf> {
        match section_path {
            ["server"] => Some(PathBuf::from("examples/server.yaml")),
            _ => None,
        }
    }
}
```

La ruta de sección por defecto es `config/<section>.yaml` para secciones
anidadas de primer nivel. Los hijos anidados se colocan bajo el stem del archivo
padre; por ejemplo, `config/trading/risk.yaml`.

## Integración CLI

Aplana `ConfigCommand` dentro de tu enum de comandos clap existente para añadir:

- `config-template`
- `config-schema`
- `config-validate`
- `completions`
- `install-completions`

La aplicación consumidora conserva su propio tipo `Parser` y su propio enum de
comandos. `rust-config-tree` solo aporta subcomandos reutilizables:

1. Añade `#[command(subcommand)] command: Command` al parser de la aplicación.
2. Añade `#[command(flatten)] Config(ConfigCommand)` al enum `Subcommand` de la
   aplicación.
3. Clap expande las variantes aplanadas en el mismo nivel de subcomandos que
   los comandos propios de la aplicación.
4. Haz match de esa variante y llama a
   `handle_config_command::<Cli, AppConfig>`.

Las flags de override de configuración específicas de la aplicación permanecen
en el parser propio de la aplicación. Por ejemplo, `--server-port` puede
mapearse a `server.port` construyendo un valor anidado
`CliOverrides { server: Some(CliServerOverrides { port }) }` y fusionándolo con
`Serialized::defaults`.

```rust
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use confique::Config;
use schemars::JsonSchema;
use rust_config_tree::{ConfigCommand, ConfigSchema, handle_config_command, load_config};

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,
    #[config(default = "paper")]
    mode: String,
}

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}

#[derive(Debug, Parser)]
#[command(name = "demo")]
struct Cli {
    #[arg(long, default_value = "config.yaml")]
    config: PathBuf,
    #[arg(long)]
    server_port: Option<u16>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Run,
    #[command(flatten)]
    Config(ConfigCommand),
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();
    match cli.command {
        Command::Run => {
            let config = load_config::<AppConfig>(&cli.config)?;
            println!("{config:#?}");
        }
        Command::Config(command) => {
            handle_config_command::<Cli, AppConfig>(command, &cli.config)?;
        }
    }

    Ok(())
}
```

`config-template --output <path>` escribe plantillas en la ruta seleccionada.
Si no se proporciona ruta de salida, escribe `config.example.yaml` en el
directorio actual. Añade `--schema <path>` para enlazar plantillas TOML y YAML
a un conjunto de JSON Schema generado sin añadir un campo `$schema` en tiempo
de ejecución. Esto también escribe el esquema raíz y los esquemas de sección en
la ruta de esquema seleccionada.

`config-schema --output <path>` escribe el JSON Schema Draft 7 raíz y los
esquemas de sección. Si no se proporciona ruta de salida, el esquema raíz se
escribe en `schemas/config.schema.json`.

`config-validate` carga el árbol completo de configuración en tiempo de
ejecución y ejecuta los valores por defecto y la validación de `confique`. Usa
los esquemas del editor para completado sin ruido mientras editas archivos
divididos; usa este comando para campos obligatorios y validación final de la
configuración. Imprime `Configuration is ok` cuando la validación tiene éxito.

`completions <shell>` imprime completions a stdout.

`install-completions <shell>` escribe completions bajo el directorio home del
usuario y actualiza el archivo de inicio del shell cuando el shell lo requiere.
Se admiten Bash, Elvish, Fish, PowerShell y Zsh.

## API de árbol de bajo nivel

Usa `load_config_tree` cuando no uses `confique` o cuando necesites acceso
directo a los resultados del recorrido:

```rust
use std::{fs, io, path::{Path, PathBuf}};

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

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let tree = load_config_tree("config.yaml", load_source)?;

    for node in tree.nodes() {
        println!("{}", node.path().display());
    }

    Ok(())
}
```

La API de árbol normaliza rutas léxicamente, rechaza rutas de include vacías,
detecta ciclos de include recursivos y omite archivos que ya se cargaron por
otra rama de include.

## Licencia

Licenciado bajo cualquiera de:

- Apache License, Version 2.0
- MIT license

a tu elección.
