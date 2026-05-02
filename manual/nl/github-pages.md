# GitHub Pages

[English](../en/github-pages.html) | [中文](../zh/github-pages.html) | [日本語](../ja/github-pages.html) | [한국어](../ko/github-pages.html) | [Français](../fr/github-pages.html) | [Deutsch](../de/github-pages.html) | [Español](../es/github-pages.html) | [Português](../pt/github-pages.html) | [Svenska](../sv/github-pages.html) | [Suomi](../fi/github-pages.html) | [Nederlands](github-pages.html)

Deze repository publiceert de handleiding met mdBook en GitHub Pages.

Elke taalhandleiding is een zelfstandig mdBook-project. Elke taal heeft een
eigen `SUMMARY.md`, zodat de linkerzijbalk alleen pagina's voor de huidige taal
bevat:

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
  ko/
  fr/
  de/
  es/
  pt/
  sv/
  fi/
  nl/
```

Bouw lokaal met:

```bash
scripts/publish-pages.sh
```

De gegenereerde site wordt geschreven naar:

```text
target/mdbook
```

## Publicatieworkflow

De workflow in `.github/workflows/pages.yml` draait bij pushes naar `main` en
bij handmatige dispatch. Hij:

1. Checkt de repository uit.
2. Installeert mdBook.
3. Draait `scripts/publish-pages.sh`.
4. Uploadt `target/mdbook` als Pages-artefact.
5. Deployt het artefact naar GitHub Pages.

De gepubliceerde URL is:

```text
https://developerworks.github.io/rust-config-tree/
```

## Crate-release

Voor de volledige flow van commit, push, Pages-deploy en crate-publicatie:

```bash
scripts/release.sh --execute --message "Release 0.1.3"
```

Gebruik de crate-releasehelper vanuit de repositoryroot:

```bash
scripts/publish-crate.sh
```

De standaardmodus voert controles en `cargo publish --dry-run` uit. Om naar
crates.io te publiceren nadat de controles slagen. Als de huidige versie al op
crates.io bestaat, verhoogt het script automatisch de patchversie:

```bash
scripts/publish-crate.sh --execute
```

Scriptgebruik wordt samengevat in `scripts/README.md`.
