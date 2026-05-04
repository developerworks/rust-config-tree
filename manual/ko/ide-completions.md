# IDE 완성

[English](../en/ide-completions.html) | [中文](../zh/ide-completions.html) | [日本語](../ja/ide-completions.html) | [한국어](ide-completions.html) | [Français](../fr/ide-completions.html) | [Deutsch](../de/ide-completions.html) | [Español](../es/ide-completions.html) | [Português](../pt/ide-completions.html) | [Svenska](../sv/ide-completions.html) | [Suomi](../fi/ide-completions.html) | [Nederlands](../nl/ide-completions.html)

생성된 JSON Schema는 TOML, YAML, JSON, JSON5 설정 파일에서 사용할 수 있습니다.
스키마는 `confique`가 사용하는 것과 같은 Rust 타입에서 생성됩니다.

```rust
use confique::Config;
use schemars::JsonSchema;

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    server: ServerConfig,
}
```

다음으로 생성합니다.

```rust
use rust_config_tree::write_config_schemas;

write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

이는 루트 스키마와 `schemas/server.schema.json` 같은 섹션 스키마를 씁니다. 생성된
스키마는 `required` 제약을 생략하므로 누락 필드 진단 없이 부분 설정 파일에서
완성이 동작합니다. 루트 스키마는 분할된 섹션 프로퍼티를 생략하므로, 자식 섹션 완성은 대응하는
섹션 스키마를 바인딩한 파일에서만 사용할 수 있습니다. 표시하지 않은 중첩 섹션은
루트 스키마에 남습니다.

`x-env-only`로 표시한 필드는 생성된 스키마에서 생략되므로, 환경 변수로만 제공해야 하는 secret이나 기타 값은 IDE가 완성하지 않습니다.

IDE 스키마는 완성과 기본 에디터 검사용입니다. 타입, enum, 알 수 없는 프로퍼티
검사처럼 생성된 스키마가 표현할 수 있는 범위만 다룹니다. 구체적인 필드 값이
애플리케이션에서 유효한지는 판단하지 않습니다. 필드 값 유효성 검사는 코드에서
`#[config(validate = Self::validate)]`로 구현하고, `load_config` 또는
`config-validate`로 실행합니다. 필수 필드와 최종 병합 설정 검증도 이 런타임
경로에서 처리합니다.

## TOML

TOML 파일은 파일 맨 위의 `#:schema` 지시문으로 스키마를 바인딩해야 합니다.

```toml
#:schema ./schemas/myapp.schema.json

[server]
bind = "0.0.0.0"
port = 3000
```

TOML에서 루트 `$schema = "..."` 필드를 사용하지 마세요. 이는 실제 설정 데이터가
되며 런타임 deserialization에 영향을 줄 수 있습니다.
`write_config_templates_with_schema`는 TOML 템플릿에 `#:schema` 지시문을 자동으로
추가합니다.

## YAML

YAML 파일은 YAML Language Server modeline을 사용해야 합니다.

```yaml
# yaml-language-server: $schema=./schemas/myapp.schema.json

server:
  bind: 0.0.0.0
  port: 3000
```

`write_config_templates_with_schema`는 YAML 템플릿에 이 modeline을 자동으로
추가합니다. 분할 YAML 템플릿은 섹션 스키마를 바인딩합니다. 예를 들어
`config/log.yaml`은 `../schemas/log.schema.json`를 바인딩합니다.

## JSON

JSON은 주석을 담을 수 없고 `$schema`는 실제 JSON 프로퍼티입니다. 런타임 설정
파일은 깨끗하게 유지하고 에디터 설정을 통해 JSON 파일을 바인딩하세요.

```json
{
  "json.schemas": [
    {
      "fileMatch": [
        "/config.json",
        "/config.*.json",
        "/deploy/*.json"
      ],
      "url": "./schemas/myapp.schema.json"
    }
  ]
}
```

YAML도 VS Code 설정으로 바인딩할 수 있습니다.

```json
{
  "yaml.schemas": {
    "./schemas/myapp.schema.json": [
      "config.yaml",
      "config.*.yaml",
      "deploy/*.yaml"
    ]
  }
}
```

최종 레이아웃은 다음과 같습니다.

```text
schemas/myapp.schema.json:
  Root file fields only

schemas/server.schema.json:
  Server section schema

config.toml:
  #:schema ./schemas/myapp.schema.json

config.yaml:
  # yaml-language-server: $schema=./schemas/myapp.schema.json

config/server.yaml:
  # yaml-language-server: $schema=../schemas/server.schema.json

config.json:
  No runtime $schema field; bind with editor settings
```

참고:

- [Tombi JSON Schema](https://tombi-toml.github.io/tombi/docs/json-schema/)
- [Taplo directives](https://taplo.tamasfe.dev/configuration/directives.html)
- [YAML Language Server](https://github.com/redhat-developer/yaml-language-server)
- [VS Code JSON](https://code.visualstudio.com/Docs/languages/json)
