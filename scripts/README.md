# Scripts

Run scripts from the repository root.

## `publish-pages.sh`

Builds the English and Chinese mdBook manuals into `target/mdbook`.

```bash
scripts/publish-pages.sh
```

GitHub Pages is deployed by `.github/workflows/pages.yml` after changes are
pushed to `main`.

## `publish-crate.sh`

Runs crate release checks and performs a `cargo publish --dry-run` by default.

```bash
scripts/publish-crate.sh
```

Publish to crates.io:

```bash
scripts/publish-crate.sh --execute
```

The script requires a clean git working tree before publishing.

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

Skip waiting for the Pages workflow:

```bash
scripts/release.sh --execute --no-wait-pages
```
