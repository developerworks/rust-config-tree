# 소개

[English](../en/introduction.html) | [中文](../zh/introduction.html) | [日本語](../ja/introduction.html) | [한국어](introduction.html) | [Français](../fr/introduction.html) | [Deutsch](../de/introduction.html) | [Español](../es/introduction.html) | [Português](../pt/introduction.html) | [Svenska](../sv/introduction.html) | [Suomi](../fi/introduction.html) | [Nederlands](../nl/introduction.html)

`rust-config-tree`는 계층형 설정 파일을 사용하는 Rust 애플리케이션을 위한
재사용 가능한 설정 트리 로딩과 CLI 헬퍼를 제공합니다.

이 crate는 작은 책임 분리를 중심으로 설계되어 있습니다.

- `confique`는 스키마 정의, 코드 기본값, 검증, 설정 템플릿 생성을 담당합니다.
- `figment`는 런타임 로딩과 런타임 소스 메타데이터를 담당합니다.
- `rust-config-tree`는 재귀 include 순회, include 경로 해석, `.env` 로딩, 템플릿 대상 발견, 재사용 가능한 clap 명령을 담당합니다.

이 crate는 애플리케이션이 다음과 같은 자연스러운 설정 파일 레이아웃을 원할 때
유용합니다.

```yaml
include:
  - config/server.yaml
  - config/database.yaml

log:
  level: info
```

포함된 각 파일은 같은 스키마 형태를 사용할 수 있고, 상대 include 경로는 그것을
선언한 파일을 기준으로 해석됩니다. 최종 설정은 여전히 일반 `confique` 스키마
값입니다.

## 주요 기능

- 순환 감지가 있는 재귀 include 순회.
- 선언 파일 기준의 상대 include 경로 해석.
- 환경 프로바이더 평가 전 `.env` 로딩.
- delimiter 분할 없는 스키마 선언 환경 변수.
- 런타임 소스 추적을 위한 Figment 메타데이터.
- `tracing`을 통한 TRACE 레벨 소스 추적 이벤트.
- 에디터 완성과 기본 schema 검사를 위한 Draft 7 JSON Schema 생성.
- 애플리케이션 코드에서 `#[config(validate = Self::validate)]`로 구현하고
  `load_config` 또는 `config-validate`로 실행하는 필드 값 유효성 검사.
- YAML, TOML, JSON, JSON5 템플릿 생성.
- 생성된 템플릿을 위한 TOML `#:schema`, YAML Language Server 스키마 modeline,
  JSON/JSON5 `$schema` 필드.
- `x-tree-split`로 표시한 중첩 섹션의 YAML 템플릿 분할.
- 설정 템플릿, JSON Schema, 셸 완성을 위한 내장 clap 하위 명령.
- `confique`를 사용하지 않는 호출자를 위한 낮은 수준의 트리 API.

## 공개 진입점

대부분의 애플리케이션에서는 다음 API를 사용하세요.

- `load_config::<S>(path)`는 최종 스키마를 로드합니다.
- `load_config_with_figment::<S>(path)`는 스키마를 로드하고 소스 추적에 사용한 Figment 그래프를 반환합니다.
- `write_config_templates::<S>(config_path, output_path)`는 루트 템플릿과 재귀적으로 발견한 자식 템플릿을 씁니다.
- `write_config_schemas::<S>(output_path)`는 루트 및 섹션 Draft 7 JSON Schema를 씁니다.
- `handle_config_command::<Cli, S>(command, config_path)`는 내장 clap 설정 명령을 처리합니다.

`confique` 없이 순회 primitive가 필요하면 `load_config_tree`를 사용하세요.
