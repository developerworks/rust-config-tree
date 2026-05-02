# Carga en tiempo de ejecución

[English](../en/runtime-loading.html) | [中文](../zh/runtime-loading.html) | [日本語](../ja/runtime-loading.html) | [한국어](../ko/runtime-loading.html) | [Français](../fr/runtime-loading.html) | [Deutsch](../de/runtime-loading.html) | [Español](runtime-loading.html) | [Português](../pt/runtime-loading.html) | [Svenska](../sv/runtime-loading.html) | [Suomi](../fi/runtime-loading.html) | [Nederlands](../nl/runtime-loading.html)

La carga en tiempo de ejecución se divide deliberadamente entre Figment y
confique:

```text
figment:
  runtime file loading
  runtime environment loading
  runtime source metadata

confique:
  schema metadata
  defaults
  validation
  config templates
```

La API principal es:

```rust
use rust_config_tree::load_config;

let config = load_config::<AppConfig>("config.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Usa `load_config_with_figment` cuando la aplicación necesita metadatos de
origen:

```rust
use rust_config_tree::load_config_with_figment;

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;
# let _ = (config, figment);
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

## Pasos de carga

El cargador de alto nivel realiza estos pasos:

1. Resolver léxicamente la ruta de configuración raíz.
2. Cargar el primer archivo `.env` encontrado al caminar hacia arriba desde el
   directorio de configuración raíz.
3. Cargar cada archivo de configuración como una capa parcial para descubrir
   includes.
4. Construir un grafo Figment desde los archivos de configuración descubiertos.
5. Fusionar `ConfiqueEnvProvider` con mayor prioridad que los archivos.
6. Fusionar opcionalmente overrides de CLI específicos de la aplicación.
7. Extraer una capa `confique` desde Figment.
8. Aplicar valores por defecto de código de `confique`.
9. Validar y construir el esquema final.

`load_config` y `load_config_with_figment` realizan los pasos 1-5 y 7-9. El
paso 6 es específico de la aplicación porque este crate no puede inferir cómo
una flag de CLI se mapea a un campo del esquema.

## Formatos de archivo

El proveedor de archivos en tiempo de ejecución se selecciona desde la extensión
de la ruta de configuración:

- `.yaml` y `.yml` usan YAML.
- `.toml` usa TOML.
- `.json` y `.json5` usan JSON.
- extensiones desconocidas o ausentes usan YAML.

La generación de plantillas sigue usando los renderizadores de plantillas de
confique para YAML, TOML y salida compatible con JSON5.

## Prioridad de includes

El cargador de alto nivel fusiona proveedores de archivo para que los archivos
incluidos tengan menor prioridad que el archivo que los incluyó. El archivo de
configuración raíz tiene la prioridad de archivo más alta.

Las variables de entorno tienen mayor prioridad que todos los archivos de
configuración. Los valores por defecto de `confique` solo se usan para valores
que no son suministrados por proveedores de tiempo de ejecución.

Cuando se fusionan overrides de CLI después de `build_config_figment`, la
precedencia completa es:

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```

La sintaxis de línea de comandos no la define `rust-config-tree`. Una flag como
`--server-port` puede sobrescribir `server.port` si la aplicación mapea ese
valor parseado a un proveedor serializado anidado. Una sintaxis con puntos como
`--server.port` o `a.b.c` solo existe si la aplicación la implementa.

Esto significa que la precedencia de CLI se aplica solo a las claves presentes
en el proveedor de overrides de la aplicación. Úsala para valores operativos
que se cambian con frecuencia en una sola ejecución. Deja la configuración
duradera en archivos.

```rust
use figment::providers::Serialized;
use serde::Serialize;
use rust_config_tree::{build_config_figment, load_config_from_figment};

#[derive(Debug, Serialize)]
struct CliOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    server: Option<CliServerOverrides>,
}

#[derive(Debug, Serialize)]
struct CliServerOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    port: Option<u16>,
}

let cli_overrides = CliOverrides {
    server: Some(CliServerOverrides { port: Some(9000) }),
};

let figment = build_config_figment::<AppConfig>("config.yaml")?
    .merge(Serialized::defaults(cli_overrides));

let config = load_config_from_figment::<AppConfig>(&figment)?;
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```
