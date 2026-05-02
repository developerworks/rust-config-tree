# Skripte

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

Fuehre Skripte aus dem Repository-Root aus.

## `publish-pages.sh`

Baut die mdBook-Handbuecher nach `target/mdbook`.

```bash
scripts/publish-pages.sh
```

GitHub Pages wird von `.github/workflows/pages.yml` bereitgestellt, nachdem
Aenderungen nach `main` gepusht wurden.

## `publish-crate.sh`

Fuehrt Release-Pruefungen fuer die Crate aus und startet standardmaessig
`cargo publish --dry-run`. Wenn `package.version` bereits auf crates.io
existiert, erhoeht das Skript automatisch die Patch-Version.

```bash
scripts/publish-crate.sh
```

Erhoehe eine andere Versionskomponente, wenn die aktuelle Version bereits
existiert:

```bash
scripts/publish-crate.sh --bump minor
scripts/publish-crate.sh --bump major
```

Auf crates.io veroeffentlichen:

```bash
scripts/publish-crate.sh --execute
```

Das Skript verlangt vor dem Veroeffentlichen einen sauberen Git-Arbeitsbaum.
Verwende `--no-bump`, um bei einer bereits existierenden Version fehlzuschlagen,
statt automatisch zu erhoehen.

Veroeffentlichungsschritte werden bei temporaeren crates.io/index-
Netzwerkfehlern wiederholt. Das Wiederholungsverhalten kann mit
Umgebungsvariablen angepasst werden:

```bash
PUBLISH_ATTEMPTS=8 PUBLISH_RETRY_DELAY=15 scripts/publish-crate.sh
```

## `release.sh`

Fuehrt den vollstaendigen Release-Ablauf aus:

1. mdBook-Pages-Artefakt bauen.
2. Rust-Pruefungen ausfuehren.
3. Code committen und pushen.
4. Auf den GitHub-Pages-Workflow warten.
5. Die Crate veroeffentlichen.

Standardmodus ist ein Dry Run:

```bash
scripts/release.sh
```

Den vollstaendigen Release ausfuehren:

```bash
scripts/release.sh --execute --message "Release 0.1.3"
```

Das vollstaendige Release-Skript bereitet die Crate-Version vor dem Commit vor,
sodass die Versionserhoehung im Release-Commit enthalten ist.

Warten auf den Pages-Workflow ueberspringen:

```bash
scripts/release.sh --execute --no-wait-pages
```
