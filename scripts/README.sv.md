# Skript

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

Kor skript fran repository-roten.

`mdbook build` fran repository-roten bygger den engelska standardmanualen.
Anvand `scripts/publish-pages.sh` for att bygga alla sprak for GitHub Pages.

## `publish-pages.sh`

Bygger alla sprakspecifika mdBook-manualer till `target/mdbook`.

```bash
scripts/publish-pages.sh
```

GitHub Pages distribueras av `.github/workflows/pages.yml` efter att andringar
har pushats till `main`.

## `publish-crate.sh`

Kor crate-releasekontroller och gor som standard en `cargo publish --dry-run`.
Om `package.version` redan finns pa crates.io hojer skriptet patch-versionen
automatiskt.

```bash
scripts/publish-crate.sh
```

Hoj en annan versionskomponent nar den aktuella versionen redan finns:

```bash
scripts/publish-crate.sh --bump minor
scripts/publish-crate.sh --bump major
```

Publicera till crates.io:

```bash
scripts/publish-crate.sh --execute
```

Skriptet kraver ett rent git-arbetstrad fore publicering. Anvand `--no-bump`
for att misslyckas i stallet for att automatiskt hoja en befintlig version.

Publiceringssteg provas om vid tillfalliga natverksfel mot crates.io/index.
Justera omforsoksbeteendet med miljovariabler:

```bash
PUBLISH_ATTEMPTS=8 PUBLISH_RETRY_DELAY=15 scripts/publish-crate.sh
```

## `release.sh`

Kor hela releaseflodet:

1. Bygg mdBook Pages-artefakten.
2. Kor Rust-kontroller.
3. Commita och pusha kod.
4. Vanta pa GitHub Pages-arbetsflodet.
5. Publicera crate.

Standardlage ar torrkorning:

```bash
scripts/release.sh
```

Kor hela releasen:

```bash
scripts/release.sh --execute --message "Release 0.1.3"
```

Det fulla releaseskriptet forbereder crate-versionen fore commit, sa
versionshojningen ingar i release-commiten.

Hoppa over vantan pa Pages-arbetsflodet:

```bash
scripts/release.sh --execute --no-wait-pages
```
