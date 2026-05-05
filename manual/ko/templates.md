# 템플릿 생성

[English](../en/templates.html) | [中文](../zh/templates.html) | [日本語](../ja/templates.html) | [한국어](templates.html) | [Français](../fr/templates.html) | [Deutsch](../de/templates.html) | [Español](../es/templates.html) | [Português](../pt/templates.html) | [Svenska](../sv/templates.html) | [Suomi](../fi/templates.html) | [Nederlands](../nl/templates.html)

템플릿은 런타임에서 사용하는 것과 같은 `confique` 스키마에서 생성됩니다.
`confique`가 doc comment, 기본값, 필수 필드, 선언된 환경 변수 이름을 포함한 실제
템플릿 내용을 렌더링합니다.

`write_config_templates`를 사용하세요.

```rust
use rust_config_tree::write_config_templates;

write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

루트 설정과 분할된 중첩 섹션의 Draft 7 JSON Schema를 생성합니다.

```rust
use rust_config_tree::write_config_schemas;

write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

중첩 필드를 자체 `*.yaml` 템플릿과 자체 `<section>.schema.json` 스키마로
생성하려면 `#[schemars(extend("x-tree-split" = true))]`를 붙입니다. 표시하지 않은
중첩 필드는 부모 템플릿과 부모 스키마에 남습니다.

값을 환경 변수로만 제공해야 하는 leaf 필드에는 `#[schemars(extend("x-env-only" = true))]`를 붙입니다. 생성된 템플릿과 JSON Schema는 env-only 필드를 생략하며, 이 생략으로 비게 된 부모 객체도 함께 제거합니다.

생성된 스키마는 `required` 제약을 생략합니다. IDE는 여전히 완성을 제공하지만
`log.yaml` 같은 부분 파일에 대해 빠진 루트 필드를 보고하지 않습니다. 루트
스키마는 루트 파일에 속하는 필드만 완성합니다. 분할된 섹션 필드는 여기서 생략되고
자체 섹션 스키마에서 완성됩니다. 존재하는 필드는 타입, enum, 알 수 없는 프로퍼티
같은 기본 에디터 검사를 받을 수 있습니다. 생성된 `*.schema.json`은 구체적인
필드 값이 애플리케이션에서 유효한지는 판단하지 않습니다. 필드 값 유효성 검사는
코드에서 `#[config(validate = Self::validate)]`로 구현하고, `load_config` 또는
`config-validate`로 실행합니다.

생성된 TOML, YAML, JSON 및 JSON5 템플릿에서 이 스키마를 바인딩합니다.

```rust
use rust_config_tree::write_config_templates_with_schema;

write_config_templates_with_schema::<AppConfig>(
    "config.toml",
    "config.example.toml",
    "schemas/myapp.schema.json",
)?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

TOML 및 YAML 루트 템플릿은 루트 스키마를 바인딩하며 자식 섹션 필드를 완성하지
않습니다. 분할 섹션 YAML 템플릿은 해당 섹션 스키마를 바인딩합니다. JSON 및 JSON5
템플릿은 VS Code가 인식할 수 있는 루트 `$schema` 필드를 받습니다. VS Code
`json.schemas`는 대체 바인딩 경로로 유지됩니다.

출력 형식은 출력 경로에서 추론됩니다.

- `.yaml` 및 `.yml`은 YAML을 생성합니다.
- `.toml`은 TOML을 생성합니다.
- `.json` 및 `.json5`는 JSON5 호환 템플릿을 생성합니다.
- 알 수 없거나 없는 확장자는 YAML을 생성합니다.

## 스키마 바인딩

스키마 경로가 `schemas/myapp.schema.json`이면 생성된 루트 템플릿은 다음을
사용합니다.

```toml
#:schema ./schemas/myapp.schema.json
```

```yaml
# yaml-language-server: $schema=./schemas/myapp.schema.json
```

생성된 섹션 템플릿은 섹션 스키마를 바인딩합니다.

```yaml
# log.yaml
# yaml-language-server: $schema=./schemas/log.schema.json
```

생성된 JSON 및 JSON5 템플릿은 VS Code가 인식하는 루트 `$schema` 필드를 씁니다.
에디터 설정은 여전히 선택 사항입니다:

```json
{
  "json.schemas": [
    {
      "fileMatch": [
        "/config.json",
        "/config.*.json"
      ],
      "url": "./schemas/myapp.schema.json"
    }
  ]
}
```

## 템플릿 소스 선택

템플릿 생성은 다음 순서로 소스 트리를 선택합니다.

1. 기존 설정 경로.
2. 기존 출력 템플릿 경로.
3. 새 빈 템플릿 트리로 취급한 출력 경로.

이를 통해 프로젝트는 현재 설정 파일에서 템플릿을 업데이트하거나, 기존 템플릿
집합을 업데이트하거나, 스키마만으로 새 템플릿 집합을 만들 수 있습니다.

## 미러링된 include 트리

소스 파일이 include를 선언하면 생성된 템플릿은 출력 디렉터리 아래에 해당 include
경로를 미러링합니다.

```yaml
# config.yaml
include:
  - server.yaml
```

`config.example.yaml`을 생성하면 다음을 씁니다.

```text
config.example.yaml
server.yaml
```

상대 include 대상은 출력 파일의 부모 디렉터리 아래에 미러링됩니다. 절대 include
대상은 절대 경로 그대로 유지됩니다.

## opt-in 섹션 분할

소스 파일에 include가 없으면 crate는 `x-tree-split`로 표시한 중첩 스키마 섹션에서 include 대상을 derive할
수 있습니다. 표시한 `server` 섹션이 있는 스키마의 경우 빈 루트 템플릿 소스는 다음을
생성할 수 있습니다.

```text
config.example.yaml
server.yaml
```

루트 템플릿은 include 블록을 받고, `server.yaml`에는 `server` 섹션만
포함됩니다. 중첩 섹션은 해당 필드도 `x-tree-split`를 가질 때만 재귀적으로 분할됩니다.
