# GitHub Pages

[English](../en/github-pages.html) | [中文](../zh/github-pages.html) | [日本語](../ja/github-pages.html) | [한국어](../ko/github-pages.html) | [Français](../fr/github-pages.html) | [Deutsch](../de/github-pages.html) | [Español](../es/github-pages.html) | [Português](github-pages.html) | [Svenska](../sv/github-pages.html) | [Suomi](../fi/github-pages.html) | [Nederlands](../nl/github-pages.html)

Este repositorio publica o manual com mdBook e GitHub Pages.

Os manuais de cada idioma sao projetos mdBook independentes. Cada idioma tem
seu proprio `SUMMARY.md`, entao a barra lateral esquerda contem apenas paginas
do idioma atual:

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
```

Construa localmente com:

```bash
scripts/publish-pages.sh
```

O site gerado e gravado em:

```text
target/mdbook
```

## Workflow de publicacao

O workflow em `.github/workflows/pages.yml` roda em pushes para `main` e por
acionamento manual. Ele:

1. Faz checkout do repositorio.
2. Instala mdBook.
3. Executa `scripts/publish-pages.sh`.
4. Envia `target/mdbook` como artefato Pages.
5. Implanta o artefato no GitHub Pages.

A URL publicada e:

```text
https://developerworks.github.io/rust-config-tree/
```

## Lancamento do crate

Para o fluxo completo de commit, push, deploy do Pages e publicacao do crate:

```bash
scripts/release.sh --execute --message "Release 0.1.3"
```

Use o auxiliar de lancamento do crate a partir da raiz do repositorio:

```bash
scripts/publish-crate.sh
```

O modo padrao executa verificacoes e `cargo publish --dry-run`. Para publicar no
crates.io depois que as verificacoes passarem. Se a versao atual ja existir no
crates.io, o script incrementa a versao patch automaticamente:

```bash
scripts/publish-crate.sh --execute
```

O uso dos scripts e resumido em `scripts/README.md`.

