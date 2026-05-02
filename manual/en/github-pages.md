# GitHub Pages

[English](github-pages.html) | [中文](../zh/github-pages.html)

This repository publishes the manual with mdBook and GitHub Pages.

The English and Chinese manuals are independent mdBook projects. Each language
has its own `SUMMARY.md`, so the left sidebar only contains pages for the
current language:

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
```

Build locally with:

```bash
scripts/publish-pages.sh
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
3. Runs `scripts/publish-pages.sh`.
4. Uploads `target/mdbook` as the Pages artifact.
5. Deploys the artifact to GitHub Pages.

The published URL is:

```text
https://developerworks.github.io/rust-config-tree/
```

## Crate Release

For the complete commit, push, Pages deploy, and crate publish flow:

```bash
scripts/release.sh --execute --message "Release 0.1.3"
```

Use the crate release helper from the repository root:

```bash
scripts/publish-crate.sh
```

The default mode runs checks and `cargo publish --dry-run`. To publish to
crates.io after the checks pass. If the current version already exists on
crates.io, the script bumps the patch version automatically:

```bash
scripts/publish-crate.sh --execute
```

Script usage is summarized in `scripts/README.md`.
