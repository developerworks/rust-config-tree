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
parciales sin diagnósticos de campos faltantes. El esquema raíz omite
propiedades de secciones anidadas, por lo que el completado de secciones hijas
solo está disponible en archivos que enlazan el esquema de sección
correspondiente.

Los esquemas del IDE siguen validando campos presentes, incluidas comprobaciones
de tipo, enum y propiedades desconocidas admitidas por el esquema generado. Usa
`config-validate` para campos obligatorios y validación final de la
configuración fusionada.

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
por ejemplo `config/log.yaml` enlaza `../schemas/log.schema.json`.

## JSON

JSON no puede llevar comentarios, y `$schema` es una propiedad JSON real. Mantén
limpios los archivos de configuración en tiempo de ejecución y enlaza archivos
JSON mediante ajustes del editor:

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
  Root file fields only

schemas/server.schema.json:
  Server section schema

config.toml:
  #:schema ./schemas/myapp.schema.json

config.yaml:
  # yaml-language-server: $schema=./schemas/myapp.schema.json

config/server.yaml:
  # yaml-language-server: $schema=../schemas/server.schema.json

config.json:
  No runtime $schema field; bind with editor settings
```

Referencias:

- [Tombi JSON Schema](https://tombi-toml.github.io/tombi/docs/json-schema/)
- [Taplo directives](https://taplo.tamasfe.dev/configuration/directives.html)
- [YAML Language Server](https://github.com/redhat-developer/yaml-language-server)
- [VS Code JSON](https://code.visualstudio.com/Docs/languages/json)
