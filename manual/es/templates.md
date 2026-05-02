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

Los esquemas generados omiten restricciones `required`. Los IDE todavía pueden
ofrecer completado, pero archivos parciales como `config/log.yaml` no informan
campos raíz faltantes. El esquema raíz solo completa campos que pertenecen al
archivo raíz; los campos de secciones anidadas se omiten allí y se completan
mediante sus propios esquemas de sección. Los campos presentes siguen siendo
comprobados por el esquema en el IDE. Los campos obligatorios y la validación
final de la configuración fusionada los gestionan `load_config` o
`config-validate`.

Enlaza esos esquemas desde plantillas TOML y YAML generadas:

```rust
use rust_config_tree::write_config_templates_with_schema;

write_config_templates_with_schema::<AppConfig>(
    "config.toml",
    "config.example.toml",
    "schemas/myapp.schema.json",
)?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Las plantillas TOML/YAML raíz enlazan el esquema raíz y no completan campos de
secciones hijas. Las plantillas YAML de sección dividida enlazan su esquema de
sección. Las plantillas JSON y JSON5 se dejan sin cambios para que la
configuración en tiempo de ejecución no contenga un campo `$schema`. Enlaza
archivos JSON con ajustes del editor como `json.schemas` de VS Code.

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
# config/log.yaml
# yaml-language-server: $schema=../schemas/log.schema.json
```

Para JSON, mantén el archivo libre de `$schema` y enlázalo con ajustes del
editor:

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
  - config/server.yaml
```

Generar `config.example.yaml` escribe:

```text
config.example.yaml
config/server.yaml
```

Los destinos de include relativos se reflejan bajo el directorio padre del
archivo de salida. Los destinos de include absolutos siguen siendo absolutos.

## División automática de secciones

Cuando un archivo fuente no tiene includes, el crate puede derivar destinos de
include desde secciones anidadas del esquema. Para un esquema con una sección
`server`, una fuente de plantilla raíz vacía puede producir:

```text
config.example.yaml
config/server.yaml
```

La plantilla raíz recibe un bloque include, y `config/server.yaml` contiene solo
la sección `server`. Las secciones anidadas se dividen recursivamente.
