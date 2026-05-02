# GitHub Pages

[English](../en/github-pages.html) | [中文](../zh/github-pages.html) | [日本語](../ja/github-pages.html) | [한국어](../ko/github-pages.html) | [Français](../fr/github-pages.html) | [Deutsch](github-pages.html) | [Español](../es/github-pages.html) | [Português](../pt/github-pages.html) | [Svenska](../sv/github-pages.html) | [Suomi](../fi/github-pages.html) | [Nederlands](../nl/github-pages.html)

Dieses Repository veroeffentlicht das Handbuch mit mdBook und GitHub Pages.

Die Handbuecher der einzelnen Sprachen sind eigenstaendige mdBook-Projekte.
Jede Sprache hat ihr eigenes `SUMMARY.md`, sodass die linke Seitenleiste nur
Seiten der aktuellen Sprache enthaelt:

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

Lokal bauen mit:

```bash
scripts/publish-pages.sh
```

Die erzeugte Site wird hier geschrieben:

```text
target/mdbook
```

## Veroeffentlichungsworkflow

Der Workflow in `.github/workflows/pages.yml` laeuft bei Pushes nach `main` und
bei manueller Ausloesung. Er:

1. Checkt das Repository aus.
2. Installiert mdBook.
3. Fuehrt `scripts/publish-pages.sh` aus.
4. Laedt `target/mdbook` als Pages-Artefakt hoch.
5. Stellt das Artefakt auf GitHub Pages bereit.

Die veroeffentlichte URL ist:

```text
https://developerworks.github.io/rust-config-tree/
```

## Crate-Release

Fuer den vollstaendigen Ablauf aus Commit, Push, Pages-Deploy und
Crate-Veroeffentlichung:

```bash
scripts/release.sh --execute --message "Release 0.1.3"
```

Verwende den Crate-Release-Helfer aus dem Repository-Root:

```bash
scripts/publish-crate.sh
```

Der Standardmodus fuehrt Pruefungen und `cargo publish --dry-run` aus. Zum
Veroeffentlichen auf crates.io nach erfolgreichen Pruefungen. Wenn die aktuelle
Version bereits auf crates.io existiert, erhoeht das Skript automatisch die
Patch-Version:

```bash
scripts/publish-crate.sh --execute
```

Die Skriptnutzung ist in `scripts/README.md` zusammengefasst.
