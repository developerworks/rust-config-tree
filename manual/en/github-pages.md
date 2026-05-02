# GitHub Pages

[English](github-pages.md) | [中文](../zh/github-pages.md)

This repository publishes the manual with mdBook and GitHub Pages.

The mdBook source lives in `manual/`, configured by `book.toml`:

```text
book.toml
manual/
  SUMMARY.md
  en/
    introduction.md
    quick-start.md
    ...
  zh/
    introduction.md
    quick-start.md
    ...
```

Build locally with:

```bash
mdbook build
```

The generated site is written to:

```text
target/mdbook
```

## Publishing Workflow

The workflow in `.github/workflows/pages.yml` runs on pushes to `main` and on
manual dispatch. It:

1. Checks out the repository.
2. Installs mdBook.
3. Builds the manual.
4. Uploads `target/mdbook` as the Pages artifact.
5. Deploys the artifact to GitHub Pages.

The published URL is:

```text
https://developerworks.github.io/rust-config-tree/
```
