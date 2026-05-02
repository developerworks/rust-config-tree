# Scripts

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

Voer scripts uit vanuit de repositoryroot.

## `publish-pages.sh`

Bouwt alle taalspecifieke mdBook-handleidingen naar `target/mdbook`.

```bash
scripts/publish-pages.sh
```

GitHub Pages wordt gedeployed door `.github/workflows/pages.yml` nadat
wijzigingen naar `main` zijn gepusht.

## `publish-crate.sh`

Voert crate-releasecontroles uit en doet standaard een `cargo publish --dry-run`.
Als `package.version` al bestaat op crates.io, verhoogt het script automatisch
de patchversie.

```bash
scripts/publish-crate.sh
```

Verhoog een ander versieonderdeel wanneer de huidige versie al bestaat:

```bash
scripts/publish-crate.sh --bump minor
scripts/publish-crate.sh --bump major
```

Publiceer naar crates.io:

```bash
scripts/publish-crate.sh --execute
```

Het script vereist een schone git-working tree voordat er wordt gepubliceerd.
Gebruik `--no-bump` om te falen in plaats van automatisch een bestaande versie
te verhogen.

Publicatiestappen worden opnieuw geprobeerd bij tijdelijke crates.io/index-
netwerkfouten. Stem retrygedrag af met omgevingsvariabelen:

```bash
PUBLISH_ATTEMPTS=8 PUBLISH_RETRY_DELAY=15 scripts/publish-crate.sh
```

## `release.sh`

Voert de volledige releaseflow uit:

1. Bouw het mdBook Pages-artefact.
2. Voer Rust-controles uit.
3. Commit en push code.
4. Wacht op de GitHub Pages-workflow.
5. Publiceer de crate.

De standaardmodus is een dry run:

```bash
scripts/release.sh
```

Voer de volledige release uit:

```bash
scripts/release.sh --execute --message "Release 0.1.3"
```

Het volledige releasescript bereidt de crateversie voor voordat er wordt
gecommit, zodat de versieverhoging in de releasecommit zit.

Sla het wachten op de Pages-workflow over:

```bash
scripts/release.sh --execute --no-wait-pages
```
