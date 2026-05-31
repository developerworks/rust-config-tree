# 빠른 시작

[English](../en/quick-start.html) | [中文](../zh/quick-start.html) | [日本語](../ja/quick-start.html) | [한국어](quick-start.html) | [Français](../fr/quick-start.html) | [Deutsch](../de/quick-start.html) | [Español](../es/quick-start.html) | [Português](../pt/quick-start.html) | [Svenska](../sv/quick-start.html) | [Suomi](../fi/quick-start.html) | [Nederlands](../nl/quick-start.html)

애플리케이션에서 사용할 crate와 스키마/런타임 라이브러리를 추가합니다.

```toml
[dependencies]
rust-config-tree = "0.2"
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

CLI 플래그 이름은 애플리케이션이 선택합니다. `ConfigOverrides` derive 매크로를
사용하여 파싱된 CLI 플래그로부터 override 프로바이더를 구축하세요:

```rust
use clap::Parser;
use rust_config_tree::{
    ConfigSchema,
    cli::ConfigOverrides,
    config::{build_config_figment, load_config_from_figment},
};

#[derive(Debug, Parser, ConfigOverrides)]
struct Cli {
    #[arg(long)]
    config: Option<std::path::PathBuf>,

    #[arg(long)]
    #[config_override(path = "server.port")]
    server_port: Option<u16>,

    #[arg(long)]
    #[config_override(path = "log.level")]
    log_level: Option<String>,
}

let cli = Cli::parse();
let figment = build_config_figment::<AppConfig>("config.yaml")?
    .merge(cli.config_overrides()?);
let config = load_config_from_figment::<AppConfig>(&figment)?;
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

`#[config_override(path = "...")]` 속성은 각 CLI 플래그를 점으로 구분된 설정
경로에 매핑합니다. 제공된 플래그만 override 값을 생성하고, 생략된 플래그는
사라집니다. override 프로바이더는 마지막에 병합되므로 제공된 플래그가 파일 및
환경 변수 값을 재정의합니다:

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```
