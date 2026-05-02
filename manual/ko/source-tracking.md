# 소스 추적

[English](../en/source-tracking.html) | [中文](../zh/source-tracking.html) | [日本語](../ja/source-tracking.html) | [한국어](source-tracking.html) | [Français](../fr/source-tracking.html) | [Deutsch](../de/source-tracking.html) | [Español](../es/source-tracking.html) | [Português](../pt/source-tracking.html) | [Svenska](../sv/source-tracking.html) | [Suomi](../fi/source-tracking.html) | [Nederlands](../nl/source-tracking.html)

런타임 로딩에 사용한 Figment 그래프를 보존하려면 `load_config_with_figment`를
사용하세요.

```rust
use rust_config_tree::load_config_with_figment;

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

반환된 Figment 값은 런타임 값의 소스 질문에 답할 수 있습니다.

```rust
if let Some(metadata) = figment.find_metadata("database.pool_size") {
    let source = metadata.interpolate(
        &figment::Profile::Default,
        &["database", "pool_size"],
    );

    println!("database.pool_size came from {source}");
}
```

`ConfiqueEnvProvider`가 제공한 값의 경우 interpolation은 스키마에 선언된 원래
환경 변수 이름을 반환합니다.

```text
database.pool_size came from APP_DATABASE_POOL_SIZE
```

## TRACE 이벤트

로더는 `tracing::trace!`로 소스 추적 이벤트를 내보냅니다. TRACE가 활성화된
경우에만 수행됩니다.

```rust
use rust_config_tree::{load_config_with_figment, trace_config_sources};

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;

// If the tracing subscriber is initialized after config loading, emit the
// same source events again after installing the subscriber.
trace_config_sources::<AppConfig>(&figment);
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

각 이벤트는 `rust_config_tree::config` target을 사용하고 다음을 포함합니다.

- `config_key`: 점으로 구분된 설정 키.
- `source`: 렌더링된 소스 메타데이터.

`confique` 기본값에서만 온 값에는 Figment 런타임 메타데이터가 없습니다. 이런
값은 `confique default or unset optional field`로 보고됩니다.
