# 빠른 시작

[English](../en/quick-start.html) | [中文](../zh/quick-start.html) | [日本語](../ja/quick-start.html) | [한국어](quick-start.html) | [Français](../fr/quick-start.html) | [Deutsch](../de/quick-start.html) | [Español](../es/quick-start.html) | [Português](../pt/quick-start.html) | [Svenska](../sv/quick-start.html) | [Suomi](../fi/quick-start.html) | [Nederlands](../nl/quick-start.html)

애플리케이션에서 사용할 crate와 스키마/런타임 라이브러리를 추가합니다.

```toml
[dependencies]
rust-config-tree = "0.1"
confique = { version = "0.4", features = ["yaml", "toml", "json5"] }
figment = { version = "0.10", features = ["yaml", "toml", "json", "env"] }
schemars = { version = "1", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
clap = { version = "4", features = ["derive"] }
```

`confique` 스키마를 정의하고 루트 타입에 `ConfigSchema`를 구현합니다.

```rust
use std::path::PathBuf;

use confique::Config;
use rust_config_tree::ConfigSchema;

#[derive(Debug, Config)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(nested)]
    server: ServerConfig,
}

#[derive(Debug, Config)]
struct ServerConfig {
    #[config(default = "127.0.0.1")]
    #[config(env = "APP_SERVER_BIND")]
    bind: String,

    #[config(default = 8080)]
    #[config(env = "APP_SERVER_PORT")]
    port: u16,
}

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}
```

설정을 로드합니다.

```rust
use rust_config_tree::load_config;

let config = load_config::<AppConfig>("config.yaml")?;
println!("{config:#?}");
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

재귀 include가 있는 루트 파일을 사용합니다.

```yaml
# config.yaml
include:
  - config/server.yaml
```

```yaml
# config/server.yaml
server:
  bind: 0.0.0.0
  port: 3000
```

기본 `load_config` 우선순위는 다음과 같습니다.

```text
environment variables
  > config files, with later merged files overriding earlier files
    > confique code defaults
```

상위 수준 API가 include를 로드할 때 루트 파일이 가장 높은 파일 우선순위를
가집니다. 포함된 파일은 더 낮은 우선순위의 값을 제공하며 기본값 또는 섹션별
파일로 사용할 수 있습니다.

명령줄 인자는 애플리케이션별이므로 `load_config`가 자동으로 읽지 않습니다.
애플리케이션에 설정 override 플래그가 있다면 `build_config_figment` 뒤에 CLI
override를 병합하세요.

CLI 플래그 이름은 애플리케이션이 선택합니다. 자동으로 `a.b.c` 설정 경로가 되지
않습니다. `--server-port` 같은 일반 clap 플래그를 선호하고, 이를 중첩 override
구조에 매핑하세요. 중첩 직렬화 형태가 override되는 설정 키를 제어합니다.

애플리케이션의 `CliOverrides` 프로바이더에 표현된 값만 설정을 override합니다.
이는 설정 파일을 편집하지 않고 한 번의 실행에서 자주 바꾸는 파라미터에
유용합니다. 안정적인 값은 설정 파일에 두어야 합니다.

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

이 방식으로 CLI override를 병합하면 전체 우선순위는 다음과 같습니다.

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```
