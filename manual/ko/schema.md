# 설정 스키마

[English](../en/schema.html) | [中文](../zh/schema.html) | [日本語](../ja/schema.html) | [한국어](schema.html) | [Français](../fr/schema.html) | [Deutsch](../de/schema.html) | [Español](../es/schema.html) | [Português](../pt/schema.html) | [Svenska](../sv/schema.html) | [Suomi](../fi/schema.html) | [Nederlands](../nl/schema.html)

애플리케이션 스키마는 일반 `confique` 설정 타입입니다. 루트 스키마는
`ConfigSchema`를 구현해야 하며, 그래야 `rust-config-tree`가 중간 `confique`
레이어에서 재귀 include를 발견할 수 있습니다.

```rust
use std::path::PathBuf;

use confique::Config;
use schemars::JsonSchema;
use rust_config_tree::ConfigSchema;

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    database: DatabaseConfig,
}

#[derive(Debug, Config, JsonSchema)]
struct DatabaseConfig {
    #[config(env = "APP_DATABASE_URL")]
    url: String,

    #[config(default = 16)]
    #[config(env = "APP_DATABASE_POOL_SIZE")]
    pool_size: u32,
}

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}
```

## Include 필드

include 필드는 어떤 이름이어도 됩니다. `rust-config-tree`는
`ConfigSchema::include_paths`를 통해서만 이 필드를 압니다.

이 필드에는 일반적으로 빈 기본값을 두어야 합니다.

```rust
#[config(default = [])]
include: Vec<PathBuf>,
```

로더는 각 파일에 대해 부분적으로 로드된 레이어를 받습니다. 이를 통해 최종 스키마가
병합 및 검증되기 전에 자식 설정 파일을 발견할 수 있습니다.

## 중첩 섹션

구조화된 섹션에는 `#[config(nested)]`를 사용하세요. 중첩 섹션은 런타임
로딩에는 항상 사용됩니다. 중첩 필드를 독립적인 `config/*.yaml` 템플릿과
`schemas/*.schema.json` 스키마로도 생성해야 할 때
`#[schemars(extend("x-tree-split" = true))]`를 추가하세요.

```rust
#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    server: ServerConfig,
}
```

자연스러운 YAML 형태는 다음과 같습니다.

```yaml
server:
  bind: 127.0.0.1
  port: 8080
```

## 템플릿 섹션 override

템플릿 소스에 include가 없으면 crate는 `x-tree-split`로 표시한 중첩 스키마 섹션에서 자식 템플릿 파일을
derive할 수 있습니다. 기본 최상위 경로는 `config/<section>.yaml`입니다.

그 경로를 `template_path_for_section`으로 override합니다.

```rust
impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }

    fn template_path_for_section(section_path: &[&str]) -> Option<PathBuf> {
        match section_path {
            ["database"] => Some(PathBuf::from("examples/database.yaml")),
            _ => None,
        }
    }
}
```
