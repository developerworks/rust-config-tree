# Scripts

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

Execute scripts a partir da raiz do repositorio.

## `publish-pages.sh`

Constroi os manuais mdBook em `target/mdbook`.

```bash
scripts/publish-pages.sh
```

O GitHub Pages e implantado por `.github/workflows/pages.yml` depois que
alteracoes sao enviadas para `main`.

## `publish-crate.sh`

Executa verificacoes de lancamento do crate e, por padrao, faz
`cargo publish --dry-run`. Se `package.version` ja existir no crates.io, o
script incrementa a versao patch automaticamente.

```bash
scripts/publish-crate.sh
```

Incremente outro componente da versao quando a versao atual ja existir:

```bash
scripts/publish-crate.sh --bump minor
scripts/publish-crate.sh --bump major
```

Publique no crates.io:

```bash
scripts/publish-crate.sh --execute
```

O script exige uma arvore de trabalho git limpa antes da publicacao. Use
`--no-bump` para falhar em vez de incrementar automaticamente uma versao
existente.

As etapas de publicacao sao tentadas novamente para falhas transitorias de rede
do crates.io/index. Ajuste o comportamento de novas tentativas com variaveis de
ambiente:

```bash
PUBLISH_ATTEMPTS=8 PUBLISH_RETRY_DELAY=15 scripts/publish-crate.sh
```

## `release.sh`

Executa o fluxo completo de lancamento:

1. Constroi o artefato mdBook Pages.
2. Executa verificacoes Rust.
3. Faz commit e push do codigo.
4. Aguarda o workflow do GitHub Pages.
5. Publica o crate.

O modo padrao e dry run:

```bash
scripts/release.sh
```

Execute o lancamento completo:

```bash
scripts/release.sh --execute --message "Release 0.1.3"
```

O script de lancamento completo prepara a versao do crate antes do commit, entao
o incremento de versao e incluido no commit de lancamento.

Pule a espera pelo workflow Pages:

```bash
scripts/release.sh --execute --no-wait-pages
```

