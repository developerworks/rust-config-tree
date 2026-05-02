# GitHub Pages

[English](../en/github-pages.html) | [中文](../zh/github-pages.html) | [日本語](github-pages.html) | [한국어](../ko/github-pages.html) | [Français](../fr/github-pages.html) | [Deutsch](../de/github-pages.html) | [Español](../es/github-pages.html) | [Português](../pt/github-pages.html) | [Svenska](../sv/github-pages.html) | [Suomi](../fi/github-pages.html) | [Nederlands](../nl/github-pages.html)

この repository は mdBook と GitHub Pages で manual を publish します。

各 language manual は独立した mdBook project です。各 language はそれぞれ
`SUMMARY.md` を持つため、left sidebar には current language の page だけが
表示されます。

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

local build:

```bash
scripts/publish-pages.sh
```

generated site は次に書かれます。

```text
target/mdbook
```

## Publishing Workflow

`.github/workflows/pages.yml` の workflow は `main` への push と manual dispatch
で実行されます。

1. repository を checkout する。
2. mdBook を install する。
3. `scripts/publish-pages.sh` を実行する。
4. `target/mdbook` を Pages artifact として upload する。
5. artifact を GitHub Pages に deploy する。

published URL:

```text
https://developerworks.github.io/rust-config-tree/
```

## Crate Release

commit、push、Pages deploy、crate publish の完全な flow:

```bash
scripts/release.sh --execute --message "Release 0.1.3"
```

repository root から crate release helper を使います。

```bash
scripts/publish-crate.sh
```

default mode は checks と `cargo publish --dry-run` を実行します。current version
が crates.io に既に存在する場合、script は patch version を自動的に bump
します。checks が通ったあと crates.io に publish します。

```bash
scripts/publish-crate.sh --execute
```

script usage は `scripts/README.md` にまとめています。
