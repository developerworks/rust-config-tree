# Completado en IDE

[English](../en/ide-completions.html) | [中文](../zh/ide-completions.html) | [日本語](../ja/ide-completions.html) | [한국어](../ko/ide-completions.html) | [Français](../fr/ide-completions.html) | [Deutsch](../de/ide-completions.html) | [Español](ide-completions.html) | [Português](../pt/ide-completions.html) | [Svenska](../sv/ide-completions.html) | [Suomi](../fi/ide-completions.html) | [Nederlands](../nl/ide-completions.html)

Los JSON Schemas generados pueden usarse con archivos de configuración TOML,
YAML, JSON y JSON5. Se generan desde el mismo tipo Rust usado por `confique`:

```rust
use confique::Config;
use schemars::JsonSchema;

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    server: ServerConfig,
}
```

Genéralos con:

```rust
use rust_config_tree::write_config_schemas;

write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Esto escribe el esquema raíz y esquemas de sección como
`schemas/server.schema.json`. Los esquemas generados omiten restricciones
`required` para que el completado funcione en archivos de configuración
parciales sin diagnósticos de campos faltantes. El esquema raíz omite propiedades de secciones divididas, por lo que el
completado de secciones hijas solo está disponible en archivos que enlazan el
esquema de sección correspondiente. Las secciones anidadas sin marca permanecen
en el esquema raíz.

Los campos marcados con `x-env-only` se omiten de los esquemas generados, por lo que los IDE no sugieren secrets u otros valores que deben venir solo de variables de entorno.

Los esquemas del IDE sirven para completado y comprobaciones básicas del
editor, como tipo, enum y propiedades desconocidas admitidas por el esquema
generado. No deciden si un valor concreto de campo es válido para la aplicación.
La validación de valores debe implementarse en código con
`#[config(validate = Self::validate)]` y ejecutarse mediante `load_config` o
`config-validate`. Los campos obligatorios y la validación final de la
configuración fusionada también usan esas rutas de ejecución.

## TOML

Los archivos TOML deberían enlazar el esquema con una directiva `#:schema` al
inicio del archivo:

```toml
#:schema ./schemas/myapp.schema.json

[server]
bind = "0.0.0.0"
port = 3000
```

No uses un campo raíz `$schema = "..."` en TOML. Se convierte en datos reales
de configuración y puede afectar la deserialización en tiempo de ejecución.
`write_config_templates_with_schema` añade automáticamente la directiva
`#:schema` para plantillas TOML.

## YAML

Los archivos YAML deberían usar la modeline de YAML Language Server:

```yaml
# yaml-language-server: $schema=./schemas/myapp.schema.json

server:
  bind: 0.0.0.0
  port: 3000
```

`write_config_templates_with_schema` añade automáticamente esta modeline para
plantillas YAML. Las plantillas YAML divididas enlazan su esquema de sección,
por ejemplo `log.yaml` enlaza `./schemas/log.schema.json`.

## JSON

Los archivos JSON y JSON5 pueden enlazar un esquema con un campo raíz
`$schema`. `write_config_templates_with_schema` lo agrega automáticamente a las
plantillas JSON y JSON5 generadas:

```json
{
  "$schema": "./schemas/myapp.schema.json"
}
```

Los ajustes del editor siguen siendo útiles si un proyecto no quiere un enlace
dentro del archivo:

```json
{
  "json.schemas": [
    {
      "fileMatch": [
        "/config.json",
        "/config.*.json",
        "/deploy/*.json"
      ],
      "url": "./schemas/myapp.schema.json"
    }
  ]
}
```

YAML también puede enlazarse mediante ajustes de VS Code:

```json
{
  "yaml.schemas": {
    "./schemas/myapp.schema.json": [
      "config.yaml",
      "config.*.yaml",
      "deploy/*.yaml"
    ]
  }
}
```

La disposición final es:

```text
schemas/myapp.schema.json:
  Solo campos del archivo raíz

schemas/server.schema.json:
  Esquema de la sección server

config.toml:
  #:schema ./schemas/myapp.schema.json

config.yaml:
  # yaml-language-server: $schema=./schemas/myapp.schema.json

server.yaml:
  # yaml-language-server: $schema=./schemas/server.schema.json

config.json:
  "$schema": "./schemas/myapp.schema.json"
```

Referencias:

- [Tombi JSON Schema](https://tombi-toml.github.io/tombi/docs/json-schema/)
- [Taplo directives](https://taplo.tamasfe.dev/configuration/directives.html)
- [YAML Language Server](https://github.com/redhat-developer/yaml-language-server)
- [VS Code JSON](https://code.visualstudio.com/Docs/languages/json)
