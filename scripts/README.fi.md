# Skriptit

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

Aja skriptit repositorion juuresta.

## `publish-pages.sh`

Rakentaa kaikki kielikohtaiset mdBook-oppaat hakemistoon `target/mdbook`.

```bash
scripts/publish-pages.sh
```

GitHub Pages julkaistaan `.github/workflows/pages.yml`-tyonkululla sen jalkeen, kun muutokset on pushattu `main`-haaraan.

## `publish-crate.sh`

Ajaa crate-julkaisun tarkistukset ja suorittaa oletuksena `cargo publish --dry-run` -ajon. Jos `package.version` on jo olemassa crates.io:ssa, skripti kasvattaa patch-version automaattisesti.

```bash
scripts/publish-crate.sh
```

Kasvata eri version osaa, kun nykyinen versio on jo olemassa:

```bash
scripts/publish-crate.sh --bump minor
scripts/publish-crate.sh --bump major
```

Julkaise crates.io:hon:

```bash
scripts/publish-crate.sh --execute
```

Skripti vaatii puhtaan git-tyopuun ennen julkaisua. Kayta `--no-bump`, jos haluat virheen automaattisen versionkasvatuksen sijaan, kun versio on jo olemassa.

Julkaisuvaiheita yritetaan uudelleen ohimenevien crates.io/index-verkkovirheiden varalta. Saada uudelleenyrityksia ymparistomuuttujilla:

```bash
PUBLISH_ATTEMPTS=8 PUBLISH_RETRY_DELAY=15 scripts/publish-crate.sh
```

## `release.sh`

Ajaa koko julkaisuvirran:

1. Rakenna mdBook Pages -artefakti.
2. Aja Rust-tarkistukset.
3. Commitoi ja pushaa koodi.
4. Odota GitHub Pages -tyonkulkua.
5. Julkaise crate.

Oletustila on kuivajo:

```bash
scripts/release.sh
```

Aja koko julkaisu:

```bash
scripts/release.sh --execute --message "Release 0.1.3"
```

Koko julkaisuskripti valmistelee crate-version ennen commitointia, joten versionkasvatus sisaltyy julkaisucommitiin.

Ohita Pages-tyonkulun odotus:

```bash
scripts/release.sh --execute --no-wait-pages
```
