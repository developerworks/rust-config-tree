# スクリプト

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

スクリプトはリポジトリ root から実行します。

## `publish-pages.sh`

すべての言語別 mdBook manual を `target/mdbook` に build します。

```bash
scripts/publish-pages.sh
```

変更が `main` に push されると、`.github/workflows/pages.yml` が GitHub Pages
を deploy します。

## `publish-crate.sh`

crate release checks を実行し、default では `cargo publish --dry-run` を実行
します。`package.version` が crates.io に既に存在する場合、script は patch
version を自動的に bump します。

```bash
scripts/publish-crate.sh
```

既存 version と衝突したときに別の version component を bump する場合:

```bash
scripts/publish-crate.sh --bump minor
scripts/publish-crate.sh --bump major
```

crates.io に publish する場合:

```bash
scripts/publish-crate.sh --execute
```

実際に publish する前に、script は clean git working tree を要求します。
既存 version の自動 bump をせず失敗させるには `--no-bump` を使います。

一時的な crates.io/index network failure に対して publish step は retry されます。
retry behavior は environment variable で調整できます。

```bash
PUBLISH_ATTEMPTS=8 PUBLISH_RETRY_DELAY=15 scripts/publish-crate.sh
```

## `release.sh`

完全な release flow を実行します。

1. mdBook Pages artifact を build する。
2. Rust checks を実行する。
3. code を commit して push する。
4. GitHub Pages workflow を待つ。
5. crate を publish する。

default mode は dry run です。

```bash
scripts/release.sh
```

完全な release を実行します。

```bash
scripts/release.sh --execute --message "Release 0.1.3"
```

full release script は commit の前に crate version を準備するため、version bump
は release commit に含まれます。

Pages workflow の待機を skip します。

```bash
scripts/release.sh --execute --no-wait-pages
```
