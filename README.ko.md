# rust-config-tree

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

`rust-config-tree`는 계층형 설정 파일을 사용하는 Rust 애플리케이션을 위한
설정 트리 로딩과 CLI 헬퍼를 제공합니다.

프로젝트 매뉴얼: <https://developerworks.github.io/rust-config-tree/>. 영어,
중국어, 일본어, 한국어 매뉴얼은 언어 전환 링크가 있는 독립 mdBook 사이트로
게시됩니다.

처리하는 기능:

- Figment 런타임 프로바이더를 통해 `confique` 스키마를 바로 사용할 수 있는 설정 객체로 로드
- `config-template`, `completions`, `install-completions` 명령 핸들러
- 에디터 완성과 검증을 위한 Draft 7 루트 및 섹션 JSON Schema 생성
- YAML, TOML, JSON, JSON5 설정 템플릿 생성
- 런타임 필드를 추가하지 않는 TOML 및 YAML 템플릿용 스키마 지시문
- 재귀 include 순회
- 환경 값 병합 전에 `.env` 로드
- Figment 메타데이터를 통한 소스 추적
- `tracing`을 통한 TRACE 레벨 소스 추적 로그
- include를 선언한 파일 기준의 상대 include 경로 해석
- 사전식 경로 정규화
- include 순환 감지
- 결정적인 순회 순서
- 미러링된 템플릿 대상 수집
- `x-tree-split`로 표시한 중첩 스키마 섹션의 YAML 템플릿 분할

애플리케이션은 `confique::Config`를 derive하고 `ConfigSchema`를 구현해
스키마의 include 필드를 노출합니다.

## Install

```toml
[dependencies]
rust-config-tree = "0.1"
confique = { version = "0.4", features = ["yaml", "toml", "json5"] }
figment = { version = "0.10", features = ["yaml", "env"] }
schemars = { version = "1", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
clap = { version = "4", features = ["derive"] }
```

## Configuration Schema

애플리케이션 스키마가 include 필드를 소유합니다. `rust-config-tree`에는 중간
`confique` 레이어에서 include를 추출하는 작은 어댑터만 필요합니다.

```rust
use std::path::PathBuf;

use confique::Config;
use schemars::JsonSchema;
use rust_config_tree::ConfigSchema;

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(default = "paper")]
    mode: String,

    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    server: ServerConfig,
}

#[derive(Debug, Config, JsonSchema)]
struct ServerConfig {
    #[config(default = 8080)]
    port: u16,
}

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}
```

상대 include 경로는 그 경로를 선언한 파일을 기준으로 해석됩니다.

```yaml
# config.yaml
include:
  - config/server.yaml

mode: shadow
```

```yaml
# config/server.yaml
server:
  port: 7777
```

`load_config`로 최종 스키마를 로드합니다.

```rust
use rust_config_tree::load_config;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = load_config::<AppConfig>("config.yaml")?;
    println!("{config:#?}");

    Ok(())
}
```

`load_config`는 Figment가 스키마에 선언된 환경 변수를 읽기 전에, 루트 설정 파일의
디렉터리에서 위로 올라가며 처음 발견한 `.env` 파일을 로드합니다. 이미 프로세스
환경에 있는 값은 보존되며 `.env` 값보다 우선합니다.

런타임 설정 로딩은 Figment를 통해 수행됩니다. `confique`는 스키마 메타데이터,
기본값, 검증, 템플릿 생성을 계속 담당합니다. 환경 변수 이름은
`#[config(env = "...")]`에서 읽습니다. 로더는 `Env::split("_")`나
`Env::split("__")`를 사용하지 않으므로, `APP_DATABASE_POOL_SIZE` 같은 변수는
`database.pool_size` 필드에 매핑될 수 있습니다.

`load_config`는 명령줄 인자를 읽지 않습니다. CLI 플래그는 애플리케이션마다 다르기
때문입니다. CLI 오버라이드는 `build_config_figment` 뒤에 프로바이더를 병합한 다음
`load_config_from_figment`로 검증해서 추가합니다.

CLI 플래그 이름은 설정 경로에서 derive되지 않습니다. `--server.port`나 `a.b.c`에
의존하지 말고 `--server-port` 또는 `--database-url` 같은 일반 애플리케이션
플래그를 사용하세요. 어떤 설정 키가 override되는지는 중첩 직렬화 override
형태가 결정합니다.

애플리케이션의 `CliOverrides` 프로바이더에 표현된 값만 설정을 override할 수
있습니다. 이는 설정 파일을 편집하는 것보다 한 번의 실행에 플래그를 바꾸는 편이
나은, 자주 조정되는 런타임 파라미터를 위한 것입니다. 안정적인 설정은 파일에 두고,
의도적인 임시 override만 CLI 플래그로 노출하세요.

```rust
use figment::providers::Serialized;
use serde::Serialize;
use rust_config_tree::{build_config_figment, load_config_from_figment};

#[derive(Debug, Serialize)]
struct CliOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<String>,
}

fn load_with_cli_overrides(cli_mode: Option<String>) -> Result<AppConfig, Box<dyn std::error::Error + Send + Sync>> {
    let cli_overrides = CliOverrides {
        mode: cli_mode,
    };

    let figment = build_config_figment::<AppConfig>("config.yaml")?
        .merge(Serialized::defaults(cli_overrides));

    let config = load_config_from_figment::<AppConfig>(&figment)?;
    Ok(config)
}
```

이 방식으로 CLI override를 병합하면 런타임 우선순위는 다음과 같습니다.

```text
command-line overrides
  > environment variables
    > config files
      > confique code defaults
```

호출자가 소스 메타데이터를 필요로 하면 `load_config_with_figment`를 사용하세요.

```rust
use rust_config_tree::load_config_with_figment;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;

    if let Some(metadata) = figment.find_metadata("mode") {
        let source = metadata.interpolate(&figment::Profile::Default, &["mode"]);
        println!("mode came from {source}");
    }

    println!("{config:#?}");

    Ok(())
}
```

로더는 `tracing::trace!`로 설정 소스 추적도 내보냅니다. 이 이벤트는
애플리케이션의 tracing subscriber에서 TRACE가 활성화된 경우에만 생성됩니다. 설정
로드 이후 tracing을 초기화한다면 subscriber 설치 후
`trace_config_sources::<AppConfig>(&figment)`를 호출하세요.

## Template Generation

템플릿은 동일한 스키마와 include 순회 규칙으로 렌더링됩니다. 출력 형식은 출력
경로에서 추론됩니다.

- `.yaml` 및 `.yml`은 YAML 생성
- `.toml`은 TOML 생성
- `.json` 및 `.json5`는 JSON5 호환 템플릿 생성
- 알 수 없거나 없는 확장자는 YAML 생성

`write_config_schemas`를 사용해 루트 설정과 분할된 중첩 섹션의 Draft 7 JSON Schema를
생성합니다. 생성된 스키마는 `required` 제약을 생략하므로, IDE가 부분 설정 파일에
대해 누락 필드 오류를 보고하지 않고 완성을 제공할 수 있습니다.

```rust
use rust_config_tree::write_config_schemas;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;

    Ok(())
}
```

Mark a nested field with `#[schemars(extend("x-tree-split" = true))]` when it
should be generated as its own `config/*.yaml` template and
`schemas/*.schema.json` schema. Unmarked nested fields stay in the parent
template and parent schema.

`server`와 `log` 섹션이 `x-tree-split`로 표시된 스키마라면 `schemas/myapp.schema.json`,
`schemas/server.schema.json`, `schemas/log.schema.json`을 씁니다. 루트 스키마에는
`include`와 루트 스칼라 필드처럼 루트 설정 파일에 속하는 필드만 포함됩니다.
분할된 섹션 프로퍼티는 의도적으로 생략되어, `server`와 `log`는 각자의 섹션 YAML
파일을 편집할 때만 완성됩니다. 표시하지 않은 중첩 섹션은 루트 스키마에 남습니다.

`write_config_templates`를 사용해 루트 템플릿과 include 트리에서 도달 가능한 모든
템플릿 파일을 생성합니다.

```rust
use rust_config_tree::write_config_templates;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;

    Ok(())
}
```

생성된 TOML 및 YAML 템플릿이 IDE 완성과 검증을 위해 해당 스키마에 바인딩되어야
한다면 `write_config_templates_with_schema`를 사용하세요.

```rust
use rust_config_tree::write_config_templates_with_schema;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_config_templates_with_schema::<AppConfig>(
        "config.toml",
        "config.example.toml",
        "schemas/myapp.schema.json",
    )?;

    Ok(())
}
```

루트 TOML/YAML 대상은 루트 스키마를 바인딩하며 자식 섹션 필드를 완성하지
않습니다. 분할된 섹션 YAML 대상은 대응하는 섹션 스키마를 바인딩합니다. 예를 들어
`config/log.yaml`은
`# yaml-language-server: $schema=../schemas/log.schema.json`를 받습니다. JSON 및
JSON5 대상은 의도적으로 `$schema` 필드를 받지 않습니다. VS Code `json.schemas`
같은 에디터 설정으로 바인딩하세요.

템플릿 생성은 다음 순서로 소스 트리를 선택합니다.

- 기존 설정 경로
- 기존 출력 템플릿 경로
- 새 빈 템플릿 트리로 간주한 출력 경로

소스 노드에 include 목록이 없으면 `rust-config-tree`는 `x-tree-split`로 표시한 중첩 `confique` 섹션에서
자식 템플릿 파일을 derive합니다. 위 스키마에서 빈 `config.example.yaml` 소스는
다음을 생성합니다.

```text
config.example.yaml
config/server.yaml
```

루트 템플릿은 `config/server.yaml` include 블록을 받습니다. `config/server.yaml`
처럼 중첩 섹션에 매핑되는 YAML 대상은 해당 섹션만 포함합니다. 더 깊은 중첩
섹션은 해당 필드도 `x-tree-split`를 가질 때만 재귀 분할됩니다.

섹션이 다른 경로에 생성되어야 한다면 `template_path_for_section`을 override하세요.

```rust
use std::path::PathBuf;

use confique::Config;
use rust_config_tree::ConfigSchema;

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }

    fn template_path_for_section(section_path: &[&str]) -> Option<PathBuf> {
        match section_path {
            ["server"] => Some(PathBuf::from("examples/server.yaml")),
            _ => None,
        }
    }
}
```

기본 섹션 경로는 최상위 중첩 섹션에 대해 `config/<section>.yaml`입니다. 중첩
자식은 부모 파일 stem 아래에 배치됩니다. 예: `config/trading/risk.yaml`.

## CLI Integration

기존 clap 명령 enum에 `ConfigCommand`를 flatten하면 다음을 추가할 수 있습니다.

- `config-template`
- `config-schema`
- `config-validate`
- `completions`
- `install-completions`

사용 애플리케이션은 자체 `Parser` 타입과 자체 명령 enum을 유지합니다.
`rust-config-tree`는 재사용 가능한 하위 명령만 제공합니다.

1. 애플리케이션 parser에 `#[command(subcommand)] command: Command`를 추가합니다.
2. 애플리케이션의 `Subcommand` enum에 `#[command(flatten)] Config(ConfigCommand)`를 추가합니다.
3. Clap은 flatten된 variant를 애플리케이션 자체 명령과 같은 하위 명령 레벨로 확장합니다.
4. 해당 variant를 match하고 `handle_config_command::<Cli, AppConfig>`를 호출합니다.

애플리케이션별 설정 override 플래그는 애플리케이션 자체 parser에 둡니다. 예를
들어 `--server-port`는 중첩된
`CliOverrides { server: Some(CliServerOverrides { port }) }` 값을 만들고
`Serialized::defaults`로 병합해서 `server.port`에 매핑할 수 있습니다.

```rust
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use confique::Config;
use schemars::JsonSchema;
use rust_config_tree::{ConfigCommand, ConfigSchema, handle_config_command, load_config};

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,
    #[config(default = "paper")]
    mode: String,
}

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}

#[derive(Debug, Parser)]
#[command(name = "demo")]
struct Cli {
    #[arg(long, default_value = "config.yaml")]
    config: PathBuf,
    #[arg(long)]
    server_port: Option<u16>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Run,
    #[command(flatten)]
    Config(ConfigCommand),
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();
    match cli.command {
        Command::Run => {
            let config = load_config::<AppConfig>(&cli.config)?;
            println!("{config:#?}");
        }
        Command::Config(command) => {
            handle_config_command::<Cli, AppConfig>(command, &cli.config)?;
        }
    }

    Ok(())
}
```

`config-template --output <path>`는 선택한 경로에 템플릿을 씁니다. 출력 경로를
제공하지 않으면 현재 디렉터리에 `config.example.yaml`을 씁니다. 런타임 `$schema`
필드를 추가하지 않고 TOML 및 YAML 템플릿을 생성된 JSON Schema 집합에
바인딩하려면 `--schema <path>`를 추가하세요. 이 옵션은 선택한 스키마 경로에 루트
스키마와 섹션 스키마도 씁니다.

`config-schema --output <path>`는 루트 Draft 7 JSON Schema와 섹션 스키마를
씁니다. 출력 경로를 제공하지 않으면 루트 스키마는
`schemas/config.schema.json`에 쓰입니다.

`config-validate`는 전체 런타임 설정 트리를 로드하고 `confique` 기본값과 검증을
실행합니다. 분할 파일 편집 중에는 노이즈 없는 완성을 위해 에디터 스키마를
사용하고, 필수 필드와 최종 설정 검증에는 이 명령을 사용하세요. 검증이 성공하면
`Configuration is ok`를 출력합니다.

`completions <shell>`은 완성을 stdout으로 출력합니다.

`install-completions <shell>`은 사용자 홈 디렉터리 아래에 완성 파일을 쓰고,
필요한 셸에서는 시작 파일을 업데이트합니다. Bash, Elvish, Fish, PowerShell,
Zsh가 지원됩니다.

## Lower-Level Tree API

`confique`를 사용하지 않거나 순회 결과에 직접 접근해야 한다면 `load_config_tree`를
사용하세요.

```rust
use std::{fs, io, path::{Path, PathBuf}};

use rust_config_tree::{ConfigSource, load_config_tree};

fn load_source(path: &Path) -> io::Result<ConfigSource<String>> {
    let content = fs::read_to_string(path)?;
    let includes = content
        .lines()
        .filter_map(|line| line.strip_prefix("include: "))
        .map(PathBuf::from)
        .collect();

    Ok(ConfigSource::new(content, includes))
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let tree = load_config_tree("config.yaml", load_source)?;

    for node in tree.nodes() {
        println!("{}", node.path().display());
    }

    Ok(())
}
```

트리 API는 경로를 사전식으로 정규화하고, 빈 include 경로를 거부하며, 재귀 include
순환을 감지하고, 다른 include branch를 통해 이미 로드된 파일을 건너뜁니다.

## License

다음 중 하나의 라이선스로 제공됩니다.

- Apache License, Version 2.0
- MIT license

선택에 따라 사용할 수 있습니다.
