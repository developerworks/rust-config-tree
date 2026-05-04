# Generación de plantillas

[English](../en/templates.html) | [中文](../zh/templates.html) | [日本語](../ja/templates.html) | [한국어](../ko/templates.html) | [Français](../fr/templates.html) | [Deutsch](../de/templates.html) | [Español](templates.html) | [Português](../pt/templates.html) | [Svenska](../sv/templates.html) | [Suomi](../fi/templates.html) | [Nederlands](../nl/templates.html)

Las plantillas se generan desde el mismo esquema `confique` usado en tiempo de
ejecución. `confique` renderiza el contenido real de la plantilla, incluidos
comentarios de documentación, valores por defecto, campos obligatorios y nombres
de variables de entorno declaradas.

Usa `write_config_templates`:

```rust
use rust_config_tree::write_config_templates;

write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Genera JSON Schemas Draft 7 para la configuración raíz y las secciones
anidadas:

```rust
use rust_config_tree::write_config_schemas;

write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Mark a nested field with `#[schemars(extend("x-tree-split" = true))]` when it
should be generated as its own `*.yaml` template and
`<section>.schema.json` schema. Unmarked nested fields stay in the parent
template and parent schema.

Marca un campo hoja con `#[schemars(extend("x-env-only" = true))]` cuando el valor debe venir solo de variables de entorno. Las plantillas generadas y los JSON Schemas omiten los campos env-only, y tambien se eliminan los objetos padre que queden vacios.

Los esquemas generados omiten restricciones `required`. Los IDE todavía pueden
ofrecer completado, pero archivos parciales como `log.yaml` no informan
campos raíz faltantes. El esquema raíz solo completa campos que pertenecen al
archivo raíz; los campos de secciones divididas se omiten allí y se completan
mediante sus propios esquemas de sección. Los campos presentes siguen siendo
comprobados de forma básica por el editor, por ejemplo tipo, enum y propiedades
desconocidas admitidas por el esquema generado. Los `*.schema.json` generados no
deciden si un valor concreto de campo es válido para la aplicación. La
validación de valores debe implementarse en código con
`#[config(validate = Self::validate)]`; `load_config` y `config-validate`
ejecutan esa validación en tiempo de ejecución.

Enlaza esos esquemas desde plantillas TOML, YAML, JSON y JSON5 generadas:

```rust
use rust_config_tree::write_config_templates_with_schema;

write_config_templates_with_schema::<AppConfig>(
    "config.toml",
    "config.example.toml",
    "schemas/myapp.schema.json",
)?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Las plantillas raíz TOML y YAML enlazan el esquema raíz y no completan campos de
secciones hijas. Las plantillas YAML de sección dividida enlazan su esquema de
sección. Las plantillas JSON y JSON5 reciben un campo raíz `$schema` que
VS Code puede reconocer. VS Code `json.schemas` sigue siendo una ruta de enlace
alternativa.

El formato de salida se infiere de la ruta de salida:

- `.yaml` y `.yml` generan YAML.
- `.toml` genera TOML.
- `.json` y `.json5` generan plantillas compatibles con JSON5.
- extensiones desconocidas o ausentes generan YAML.

## Enlaces de esquema

Con una ruta de esquema `schemas/myapp.schema.json`, las plantillas raíz
generadas usan:

```toml
#:schema ./schemas/myapp.schema.json
```

```yaml
# yaml-language-server: $schema=./schemas/myapp.schema.json
```

Las plantillas de sección generadas enlazan esquemas de sección:

```yaml
# log.yaml
# yaml-language-server: $schema=./schemas/log.schema.json
```

Las plantillas JSON y JSON5 generadas escriben un campo raíz `$schema` que
VS Code reconoce. Los ajustes del editor siguen siendo opcionales:

```json
{
  "json.schemas": [
    {
      "fileMatch": [
        "/config.json",
        "/config.*.json"
      ],
      "url": "./schemas/myapp.schema.json"
    }
  ]
}
```

## Selección de fuente de plantilla

La generación de plantillas elige su árbol fuente en este orden:

1. Ruta de configuración existente.
2. Ruta de plantilla de salida existente.
3. Ruta de salida tratada como un nuevo árbol de plantillas vacío.

Esto permite a un proyecto actualizar plantillas desde archivos de
configuración actuales, actualizar un conjunto de plantillas existente o crear
un conjunto nuevo solo desde el esquema.

## Árboles de includes reflejados

Si el archivo fuente declara includes, las plantillas generadas reflejan esas
rutas de include bajo el directorio de salida.

```yaml
# config.yaml
include:
  - server.yaml
```

Generar `config.example.yaml` escribe:

```text
config.example.yaml
server.yaml
```

Los destinos de include relativos se reflejan bajo el directorio padre del
archivo de salida. Los destinos de include absolutos siguen siendo absolutos.

## División opt-in de secciones

Cuando un archivo fuente no tiene includes, el crate puede derivar destinos de
include desde secciones anidadas del esquema marcadas con `x-tree-split`. Para un esquema con una sección marcada
`server`, una fuente de plantilla raíz vacía puede producir:

```text
config.example.yaml
server.yaml
```

La plantilla raíz recibe un bloque include, y `server.yaml` contiene solo
la sección `server`. Las secciones anidadas solo se dividen recursivamente cuando esos campos tambien llevan `x-tree-split`.
