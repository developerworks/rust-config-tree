# GitHub Pages

[English](../en/github-pages.html) | [中文](../zh/github-pages.html) | [日本語](../ja/github-pages.html) | [한국어](../ko/github-pages.html) | [Français](../fr/github-pages.html) | [Deutsch](../de/github-pages.html) | [Español](../es/github-pages.html) | [Português](../pt/github-pages.html) | [Svenska](github-pages.html) | [Suomi](../fi/github-pages.html) | [Nederlands](../nl/github-pages.html)

Detta repository publicerar manualen med mdBook och GitHub Pages.

Manualerna ar fristaende mdBook-projekt. Varje sprak har sin egen `SUMMARY.md`,
sa vanster sidofalt innehaller bara sidor for aktuellt sprak:

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

Bygg lokalt med:

```bash
scripts/publish-pages.sh
```

Den genererade webbplatsen skrivs till:

```text
target/mdbook
```

## Publiceringsarbetsflode

Arbetsflodet i `.github/workflows/pages.yml` kors vid pushar till `main` och
vid manuell dispatch. Det:

1. Checkar ut repositoryt.
2. Installerar mdBook.
3. Kor `scripts/publish-pages.sh`.
4. Laddar upp `target/mdbook` som Pages-artefakt.
5. Distribuerar artefakten till GitHub Pages.

Den publicerade URL:en ar:

```text
https://developerworks.github.io/rust-config-tree/
```

## Crate-release

For hela flodet med commit, push, Pages-distribution och crate-publicering:

```bash
scripts/release.sh --execute --message "Release 0.1.3"
```

Anvand crate-releasehjalparen fran repository-roten:

```bash
scripts/publish-crate.sh
```

Standardlaget kor kontroller och `cargo publish --dry-run`. For att publicera
till crates.io efter att kontrollerna passerat. Om aktuell version redan finns
pa crates.io hojer skriptet patch-versionen automatiskt:

```bash
scripts/publish-crate.sh --execute
```

Skriptanvandning sammanfattas i `scripts/README.md`.
