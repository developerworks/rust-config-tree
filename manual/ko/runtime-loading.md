# 런타임 로딩

[English](../en/runtime-loading.html) | [中文](../zh/runtime-loading.html) | [日本語](../ja/runtime-loading.html) | [한국어](runtime-loading.html) | [Français](../fr/runtime-loading.html) | [Deutsch](../de/runtime-loading.html) | [Español](../es/runtime-loading.html) | [Português](../pt/runtime-loading.html) | [Svenska](../sv/runtime-loading.html) | [Suomi](../fi/runtime-loading.html) | [Nederlands](../nl/runtime-loading.html)

런타임 로딩은 의도적으로 Figment와 confique 사이에 나뉩니다.

```text
figment:
  runtime file loading
  runtime environment loading
  runtime source metadata

confique:
  schema metadata
  defaults
  validation
  config templates
```

주요 API는 다음과 같습니다.

```rust
use rust_config_tree::load_config;

let config = load_config::<AppConfig>("config.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

애플리케이션이 소스 메타데이터를 필요로 하면 `load_config_with_figment`를
사용하세요.

```rust
use rust_config_tree::load_config_with_figment;

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;
# let _ = (config, figment);
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

## 로딩 단계

상위 수준 로더는 다음 단계를 수행합니다.

1. 루트 설정 경로를 사전식으로 해석합니다.
2. 루트 설정 디렉터리에서 위로 올라가며 처음 발견한 `.env` 파일을 로드합니다.
3. 각 설정 파일을 부분 레이어로 로드해 include를 발견합니다.
4. 발견한 설정 파일에서 Figment 그래프를 빌드합니다.
5. `ConfiqueEnvProvider`를 파일보다 높은 우선순위로 병합합니다.
6. 선택적으로 애플리케이션별 CLI override를 병합합니다.
7. Figment에서 `confique` 레이어를 추출합니다.
8. `confique` 코드 기본값을 적용합니다.
9. 최종 스키마를 검증하고 구성합니다.

`load_config`와 `load_config_with_figment`는 1-5단계와 7-9단계를 수행합니다.
6단계는 CLI 플래그가 스키마 필드에 어떻게 매핑되는지 이 crate가 추론할 수 없기
때문에 애플리케이션별입니다.

## 파일 형식

런타임 파일 프로바이더는 설정 경로 확장자에서 선택됩니다.

- `.yaml` 및 `.yml`은 YAML을 사용합니다.
- `.toml`은 TOML을 사용합니다.
- `.json` 및 `.json5`는 JSON을 사용합니다.
- 알 수 없거나 없는 확장자는 YAML을 사용합니다.

템플릿 생성은 YAML, TOML, JSON5 호환 출력에 대해 여전히 confique의 템플릿
렌더러를 사용합니다.

## Include 우선순위

상위 수준 로더는 포함된 파일이 그 파일을 include한 파일보다 낮은 우선순위가
되도록 파일 프로바이더를 병합합니다. 루트 설정 파일이 가장 높은 파일 우선순위를
가집니다.

환경 변수는 모든 설정 파일보다 높은 우선순위를 가집니다. `confique` 기본값은
런타임 프로바이더가 제공하지 않은 값에만 사용됩니다.

`build_config_figment` 뒤에 CLI override를 병합하면 전체 우선순위는 다음과
같습니다.

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```

명령줄 문법은 `rust-config-tree`가 정의하지 않습니다. 애플리케이션이 파싱한 값을
중첩 직렬화 프로바이더에 매핑하면 `--server-port` 같은 플래그가 `server.port`를
override할 수 있습니다. 점이 있는 `--server.port` 또는 `a.b.c` 문법은
애플리케이션이 구현한 경우에만 존재합니다.

즉 CLI 우선순위는 애플리케이션의 override 프로바이더에 있는 키에만 적용됩니다.
한 번의 실행에서 자주 바뀌는 운영 값을 위해 사용하고, 지속적인 설정은 파일에
남겨두세요.

```rust
use figment::providers::Serialized;
use serde::Serialize;
use rust_config_tree::{build_config_figment, load_config_from_figment};

#[derive(Debug, Serialize)]
struct CliOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    server: Option<CliServerOverrides>,
}

#[derive(Debug, Serialize)]
struct CliServerOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    port: Option<u16>,
}

let cli_overrides = CliOverrides {
    server: Some(CliServerOverrides { port: Some(9000) }),
};

let figment = build_config_figment::<AppConfig>("config.yaml")?
    .merge(Serialized::defaults(cli_overrides));

let config = load_config_from_figment::<AppConfig>(&figment)?;
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```
