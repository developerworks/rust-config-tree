# GitHub Pages

[English](../en/github-pages.html) | [中文](../zh/github-pages.html) | [日本語](../ja/github-pages.html) | [한국어](github-pages.html) | [Français](../fr/github-pages.html) | [Deutsch](../de/github-pages.html) | [Español](../es/github-pages.html) | [Português](../pt/github-pages.html) | [Svenska](../sv/github-pages.html) | [Suomi](../fi/github-pages.html) | [Nederlands](../nl/github-pages.html)

이 저장소는 mdBook과 GitHub Pages로 매뉴얼을 게시합니다.

언어별 매뉴얼은 독립 mdBook 프로젝트입니다. 각 언어에는 자체 `SUMMARY.md`가
있으므로 왼쪽 사이드바에는 현재 언어의 페이지만 포함됩니다.

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

로컬에서 빌드합니다.

```bash
scripts/publish-pages.sh
```

생성된 사이트는 다음 위치에 쓰입니다.

```text
target/mdbook
```

## 게시 workflow

`.github/workflows/pages.yml`의 workflow는 `main` push와 수동 dispatch에서
실행됩니다. 이 workflow는 다음을 수행합니다.

1. 저장소를 checkout합니다.
2. mdBook을 설치합니다.
3. `scripts/publish-pages.sh`를 실행합니다.
4. `target/mdbook`을 Pages artifact로 업로드합니다.
5. artifact를 GitHub Pages에 배포합니다.

게시 URL은 다음과 같습니다.

```text
https://developerworks.github.io/rust-config-tree/
```

## Crate 릴리스

commit, push, Pages 배포, crate 게시를 포함한 전체 흐름:

```bash
scripts/release.sh --execute --message "Release 0.1.3"
```

저장소 루트에서 crate 릴리스 헬퍼를 사용하세요.

```bash
scripts/publish-crate.sh
```

기본 모드는 검사와 `cargo publish --dry-run`을 실행합니다. 검사가 통과한 뒤
crates.io에 게시하려면 다음을 사용합니다. 현재 버전이 이미 crates.io에 있으면
스크립트가 patch 버전을 자동으로 올립니다.

```bash
scripts/publish-crate.sh --execute
```

스크립트 사용법은 `scripts/README.md`에 요약되어 있습니다.
