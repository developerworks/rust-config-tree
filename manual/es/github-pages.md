# GitHub Pages

[English](../en/github-pages.html) | [中文](../zh/github-pages.html) | [日本語](../ja/github-pages.html) | [한국어](../ko/github-pages.html) | [Français](../fr/github-pages.html) | [Deutsch](../de/github-pages.html) | [Español](github-pages.html) | [Português](../pt/github-pages.html) | [Svenska](../sv/github-pages.html) | [Suomi](../fi/github-pages.html) | [Nederlands](../nl/github-pages.html)

Este repositorio publica el manual con mdBook y GitHub Pages.

Los manuales por idioma son proyectos mdBook independientes. Cada idioma tiene
su propio `SUMMARY.md`, por lo que la barra lateral izquierda solo contiene
páginas del idioma actual:

```text
manual/
  en/
    book.toml
    SUMMARY.md
    introduction.md
    quick-start.md
    ...
  zh/
    book.toml
    SUMMARY.md
    introduction.md
    quick-start.md
    ...
  ja/
    book.toml
    SUMMARY.md
    introduction.md
    quick-start.md
    ...
```

Construye localmente con:

```bash
scripts/publish-pages.sh
```

El sitio generado se escribe en:

```text
target/mdbook
```

## Workflow de publicación

El workflow en `.github/workflows/pages.yml` se ejecuta en pushes a `main` y en
manual dispatch. Hace lo siguiente:

1. Hace checkout del repositorio.
2. Instala mdBook.
3. Ejecuta `scripts/publish-pages.sh`.
4. Sube `target/mdbook` como artefacto de Pages.
5. Despliega el artefacto en GitHub Pages.

La URL publicada es:

```text
https://developerworks.github.io/rust-config-tree/
```

## Release del crate

Para el flujo completo de commit, push, despliegue de Pages y publicación del
crate:

```bash
scripts/release.sh --execute --message "Release 0.1.3"
```

Usa el ayudante de release del crate desde la raíz del repositorio:

```bash
scripts/publish-crate.sh
```

El modo por defecto ejecuta comprobaciones y `cargo publish --dry-run`. Para
publicar en crates.io después de que las comprobaciones pasen. Si la versión
actual ya existe en crates.io, el script incrementa automáticamente la versión
patch:

```bash
scripts/publish-crate.sh --execute
```

El uso de scripts se resume en `scripts/README.md`.
