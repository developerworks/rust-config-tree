# Scripts

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

스크립트는 저장소 루트에서 실행하세요.

## `publish-pages.sh`

영어, 중국어, 일본어, 한국어 mdBook 매뉴얼을 `target/mdbook`에 빌드합니다.

```bash
scripts/publish-pages.sh
```

변경 사항이 `main`에 push된 뒤 `.github/workflows/pages.yml`이 GitHub Pages를
배포합니다.

## `publish-crate.sh`

crate 릴리스 검사를 실행하고 기본적으로 `cargo publish --dry-run`을 수행합니다.
`package.version`이 이미 crates.io에 있으면 스크립트가 patch 버전을 자동으로
올립니다.

```bash
scripts/publish-crate.sh
```

현재 버전이 이미 존재할 때 다른 버전 컴포넌트를 올립니다.

```bash
scripts/publish-crate.sh --bump minor
scripts/publish-crate.sh --bump major
```

crates.io에 게시합니다.

```bash
scripts/publish-crate.sh --execute
```

스크립트는 게시 전에 깨끗한 git working tree를 요구합니다. 기존 버전 자동 bump
대신 실패하게 하려면 `--no-bump`를 사용하세요.

일시적인 crates.io/index 네트워크 실패에 대해 게시 단계는 재시도됩니다. 재시도
동작은 환경 변수로 조정합니다.

```bash
PUBLISH_ATTEMPTS=8 PUBLISH_RETRY_DELAY=15 scripts/publish-crate.sh
```

## `release.sh`

전체 릴리스 흐름을 실행합니다.

1. mdBook Pages artifact를 빌드합니다.
2. Rust 검사를 실행합니다.
3. 코드를 commit하고 push합니다.
4. GitHub Pages workflow를 기다립니다.
5. crate를 게시합니다.

기본 모드는 dry run입니다.

```bash
scripts/release.sh
```

전체 릴리스를 실행합니다.

```bash
scripts/release.sh --execute --message "Release 0.1.3"
```

전체 릴리스 스크립트는 commit 전에 crate 버전을 준비하므로, 버전 bump가 릴리스
commit에 포함됩니다.

Pages workflow 대기를 건너뜁니다.

```bash
scripts/release.sh --execute --no-wait-pages
```
