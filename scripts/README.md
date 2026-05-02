# Scripts

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

Run scripts from the repository root.

`mdbook build` from the repository root builds the default English manual. Use
`scripts/publish-pages.sh` to build every language for GitHub Pages.

## `publish-pages.sh`

Builds all language-specific mdBook manuals into `target/mdbook`.

```bash
scripts/publish-pages.sh
```

GitHub Pages is deployed by `.github/workflows/pages.yml` after changes are
pushed to `main`.

## `publish-crate.sh`

Runs crate release checks and performs a `cargo publish --dry-run` by default.
If `package.version` already exists on crates.io, the script bumps the patch
version automatically.

```bash
scripts/publish-crate.sh
```

Bump a different version component when the current version already exists:

```bash
scripts/publish-crate.sh --bump minor
scripts/publish-crate.sh --bump major
```

Publish to crates.io:

```bash
scripts/publish-crate.sh --execute
```

The script requires a clean git working tree before publishing. Use `--no-bump`
to fail instead of auto-bumping an existing version.

Publish steps are retried for transient crates.io/index network failures. Tune
retry behavior with environment variables:

```bash
PUBLISH_ATTEMPTS=8 PUBLISH_RETRY_DELAY=15 scripts/publish-crate.sh
```

## `release.sh`

Runs the full release flow:

1. Build the mdBook Pages artifact.
2. Run Rust checks.
3. Commit and push code.
4. Wait for the GitHub Pages workflow.
5. Publish the crate.

Default mode is a dry run:

```bash
scripts/release.sh
```

Execute the full release:

```bash
scripts/release.sh --execute --message "Release 0.1.3"
```

The full release script prepares the crate version before committing, so the
version bump is included in the release commit.

Skip waiting for the Pages workflow:

```bash
scripts/release.sh --execute --no-wait-pages
```
