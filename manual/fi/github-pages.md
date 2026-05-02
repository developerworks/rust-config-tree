# GitHub Pages

[English](../en/github-pages.html) | [中文](../zh/github-pages.html) | [日本語](../ja/github-pages.html) | [한국어](../ko/github-pages.html) | [Français](../fr/github-pages.html) | [Deutsch](../de/github-pages.html) | [Español](../es/github-pages.html) | [Português](../pt/github-pages.html) | [Svenska](../sv/github-pages.html) | [Suomi](github-pages.html) | [Nederlands](../nl/github-pages.html)

Tama repository julkaisee oppaan mdBookilla ja GitHub Pagesilla.

Jokainen kieliopas on itsenainen mdBook-projekti. Jokaisella kielella on oma `SUMMARY.md`, joten vasen sivupalkki sisaltaa vain nykyisen kielen sivut:

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

Rakenna paikallisesti:

```bash
scripts/publish-pages.sh
```

Luotu sivusto kirjoitetaan hakemistoon:

```text
target/mdbook
```

## Julkaisun tyonkulku

`.github/workflows/pages.yml`-tyonkulku ajetaan pusheissa `main`-haaraan ja kasin kaynnistettyna. Se:

1. Checkouttaa repositorion.
2. Asentaa mdBookin.
3. Ajaa `scripts/publish-pages.sh`.
4. Lataa `target/mdbook`-hakemiston Pages-artefaktiksi.
5. Julkaisee artefaktin GitHub Pagesiin.

Julkaistu URL on:

```text
https://developerworks.github.io/rust-config-tree/
```

## Crate-julkaisu

Koko commit-, push-, Pages-julkaisu- ja crate-julkaisuvirralle:

```bash
scripts/release.sh --execute --message "Release 0.1.3"
```

Kayta crate-julkaisun apuria repositorion juuresta:

```bash
scripts/publish-crate.sh
```

Oletustila ajaa tarkistukset ja `cargo publish --dry-run` -komennon. Julkaise crates.io:hon tarkistusten onnistuttua. Jos nykyinen versio on jo olemassa crates.io:ssa, skripti kasvattaa patch-version automaattisesti:

```bash
scripts/publish-crate.sh --execute
```

Skriptin kaytto on tiivistetty tiedostossa `scripts/README.md`.
