# Introducción

[English](../en/introduction.html) | [中文](../zh/introduction.html) | [日本語](../ja/introduction.html) | [한국어](../ko/introduction.html) | [Français](../fr/introduction.html) | [Deutsch](../de/introduction.html) | [Español](introduction.html) | [Português](../pt/introduction.html) | [Svenska](../sv/introduction.html) | [Suomi](../fi/introduction.html) | [Nederlands](../nl/introduction.html)

`rust-config-tree` proporciona carga reutilizable de árboles de configuración y
ayudantes de CLI para aplicaciones Rust que usan archivos de configuración por
capas.

El crate está diseñado alrededor de una pequeña división de responsabilidades:

- `confique` posee las definiciones de esquema, valores por defecto en código,
  validación y generación de plantillas de configuración.
- `figment` posee la carga en tiempo de ejecución y los metadatos de origen en
  tiempo de ejecución.
- `rust-config-tree` posee el recorrido recursivo de includes, la resolución de
  rutas de include, la carga de `.env`, el descubrimiento de destinos de
  plantilla y comandos clap reutilizables.

El crate es útil cuando una aplicación quiere una distribución natural de
archivos de configuración como esta:

```yaml
include:
  - config/server.yaml
  - config/database.yaml

log:
  level: info
```

Cada archivo incluido puede usar la misma forma de esquema, y las rutas de
include relativas se resuelven desde el archivo que las declaró. La
configuración final sigue siendo un valor normal de esquema `confique`.

## Funciones principales

- Recorrido recursivo de includes con detección de ciclos.
- Rutas de include relativas resueltas desde el archivo que las declara.
- Carga de `.env` antes de evaluar proveedores de entorno.
- Variables de entorno declaradas por esquema sin división por delimitadores.
- Metadatos de Figment para seguimiento de origen en tiempo de ejecución.
- Eventos de seguimiento de origen en nivel TRACE mediante `tracing`.
- Generación de JSON Schema Draft 7 para completado y comprobaciones básicas de esquema en editores.
- Validación de valores de campo en código de aplicación con
  `#[config(validate = Self::validate)]`, ejecutada por `load_config` o
  `config-validate`.
- Generación de plantillas YAML, TOML, JSON y JSON5.
- Directivas TOML `#:schema`, modelines de YAML Language Server y campos
  JSON/JSON5 `$schema` para plantillas generadas.
- División opt-in de plantillas YAML para secciones marcadas con `x-tree-split`.
- Subcomandos clap incorporados para plantillas de configuración, JSON Schema y
  completions de shell.
- Una API de árbol de menor nivel para llamadores que no usan `confique`.

## Puntos de entrada públicos

Usa estas APIs para la mayoría de aplicaciones:

- `load_config::<S>(path)` carga el esquema final.
- `load_config_with_figment::<S>(path)` carga el esquema y devuelve el grafo
  Figment usado para seguimiento de origen.
- `write_config_templates::<S>(config_path, output_path)` escribe la plantilla
  raíz y las plantillas hijas descubiertas recursivamente.
- `write_config_schemas::<S>(output_path)` escribe JSON Schemas Draft 7 raíz y
  de sección.
- `handle_config_command::<Cli, S>(command, config_path)` maneja comandos clap
  de configuración incorporados.

Usa `load_config_tree` cuando necesites la primitiva de recorrido sin
`confique`.
