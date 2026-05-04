# GitHub Pages

[English](../en/github-pages.html) | [中文](../zh/github-pages.html) | [日本語](../ja/github-pages.html) | [한국어](../ko/github-pages.html) | [Français](github-pages.html) | [Deutsch](../de/github-pages.html) | [Español](../es/github-pages.html) | [Português](../pt/github-pages.html) | [Svenska](../sv/github-pages.html) | [Suomi](../fi/github-pages.html) | [Nederlands](../nl/github-pages.html)

Ce depot publie le manuel avec mdBook et GitHub Pages.

Les manuels de chaque langue sont des projets mdBook independants. Chaque
langue a son propre `SUMMARY.md`, donc la barre laterale gauche ne contient que
les pages de la langue courante :

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

Construisez localement avec :

```bash
scripts/publish-pages.sh
```

Le site genere est ecrit dans :

```text
target/mdbook
```

## Workflow de publication

Le workflow dans `.github/workflows/pages.yml` s'execute lors des pushes vers
`main` et en declenchement manuel. Il :

1. Recupere le depot.
2. Installe mdBook.
3. Execute `scripts/publish-pages.sh`.
4. Televerse `target/mdbook` comme artefact Pages.
5. Deploie l'artefact vers GitHub Pages.

L'URL publiee est :

```text
https://developerworks.github.io/rust-config-tree/
```

## Publication de la crate

Pour le flux complet de commit, push, deploiement Pages et publication de la
crate :

```bash
scripts/release.sh --execute --message "Release 0.1.3"
```

Utilisez l'assistant de publication de crate depuis la racine du depot :

```bash
scripts/publish-crate.sh
```

Le mode par defaut lance les controles et `cargo publish --dry-run`. Pour
publier sur crates.io apres la reussite des controles. Si la version courante
existe deja sur crates.io, le script incremente automatiquement la version
patch :

```bash
scripts/publish-crate.sh --execute
```

L'utilisation des scripts est resumee dans `scripts/README.md`.
