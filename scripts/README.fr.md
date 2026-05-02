# Scripts

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

Executez les scripts depuis la racine du depot.

`mdbook build` depuis la racine du depot construit le manuel anglais par defaut.
Utilisez `scripts/publish-pages.sh` pour construire toutes les langues de
GitHub Pages.

## `publish-pages.sh`

Construit tous les manuels mdBook par langue dans `target/mdbook`.

```bash
scripts/publish-pages.sh
```

GitHub Pages est deploye par `.github/workflows/pages.yml` apres l'envoi des
changements vers `main`.

## `publish-crate.sh`

Lance les controles de publication de la crate et effectue par defaut un
`cargo publish --dry-run`. Si `package.version` existe deja sur crates.io, le
script incremente automatiquement la version patch.

```bash
scripts/publish-crate.sh
```

Incrementez un autre composant de version lorsque la version courante existe
deja :

```bash
scripts/publish-crate.sh --bump minor
scripts/publish-crate.sh --bump major
```

Publier sur crates.io :

```bash
scripts/publish-crate.sh --execute
```

Le script exige un arbre git propre avant la publication. Utilisez `--no-bump`
pour echouer au lieu d'incrementer automatiquement une version existante.

Les etapes de publication sont retentees pour les echecs reseau transitoires de
crates.io ou de l'index. Ajustez ce comportement avec des variables
d'environnement :

```bash
PUBLISH_ATTEMPTS=8 PUBLISH_RETRY_DELAY=15 scripts/publish-crate.sh
```

## `release.sh`

Lance le flux de publication complet :

1. Construire l'artefact mdBook Pages.
2. Executer les controles Rust.
3. Commiter et pousser le code.
4. Attendre le workflow GitHub Pages.
5. Publier la crate.

Le mode par defaut est un dry run :

```bash
scripts/release.sh
```

Executer la publication complete :

```bash
scripts/release.sh --execute --message "Release 0.1.3"
```

Le script de publication complet prepare la version de la crate avant le
commit, donc l'incrementation de version est incluse dans le commit de
publication.

Ignorer l'attente du workflow Pages :

```bash
scripts/release.sh --execute --no-wait-pages
```
