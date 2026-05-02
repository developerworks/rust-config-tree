# Scripts

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

Ejecuta los scripts desde la raíz del repositorio.

## `publish-pages.sh`

Construye los manuales mdBook en `target/mdbook`.

```bash
scripts/publish-pages.sh
```

GitHub Pages se despliega mediante `.github/workflows/pages.yml` después de que
los cambios se envían a `main`.

## `publish-crate.sh`

Ejecuta las comprobaciones de publicación del crate y realiza
`cargo publish --dry-run` por defecto. Si `package.version` ya existe en
crates.io, el script incrementa automáticamente la versión patch.

```bash
scripts/publish-crate.sh
```

Incrementa un componente de versión distinto cuando la versión actual ya
existe:

```bash
scripts/publish-crate.sh --bump minor
scripts/publish-crate.sh --bump major
```

Publica en crates.io:

```bash
scripts/publish-crate.sh --execute
```

El script requiere un árbol git limpio antes de publicar. Usa `--no-bump` para
fallar en lugar de incrementar automáticamente una versión existente.

Los pasos de publicación se reintentan ante fallos transitorios de red de
crates.io/index. Ajusta el comportamiento de reintentos con variables de
entorno:

```bash
PUBLISH_ATTEMPTS=8 PUBLISH_RETRY_DELAY=15 scripts/publish-crate.sh
```

## `release.sh`

Ejecuta el flujo completo de release:

1. Construir el artefacto mdBook Pages.
2. Ejecutar comprobaciones Rust.
3. Crear commit y hacer push del código.
4. Esperar el workflow de GitHub Pages.
5. Publicar el crate.

El modo por defecto es una ejecución dry run:

```bash
scripts/release.sh
```

Ejecuta el release completo:

```bash
scripts/release.sh --execute --message "Release 0.1.3"
```

El script de release completo prepara la versión del crate antes del commit, de
modo que el incremento de versión queda incluido en el commit de release.

Omite la espera del workflow de Pages:

```bash
scripts/release.sh --execute --no-wait-pages
```
