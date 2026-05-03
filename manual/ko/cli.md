# CLI 통합

[English](../en/cli.html) | [中文](../zh/cli.html) | [日本語](../ja/cli.html) | [한국어](cli.html) | [Français](../fr/cli.html) | [Deutsch](../de/cli.html) | [Español](../es/cli.html) | [Português](../pt/cli.html) | [Svenska](../sv/cli.html) | [Suomi](../fi/cli.html) | [Nederlands](../nl/cli.html)

`ConfigCommand`는 재사용 가능한 clap 하위 명령을 제공합니다.

- `config-template`
- `config-schema`
- `config-validate`
- `completions`
- `install-completions`
- `uninstall-completions`

이 내장 하위 명령은 애플리케이션별 설정 override 플래그와 분리되어 있습니다.
설정 override 플래그는 런타임 로딩 경로에서 Figment 프로바이더로 병합하세요.

설정 override 플래그는 사용 애플리케이션 CLI의 일부로 남습니다. 이름이 점으로
구분된 설정 경로와 일치할 필요는 없습니다. 예를 들어 애플리케이션은
`--server-port`를 파싱하고 중첩 `server.port` 설정 키에 매핑할 수 있습니다.
애플리케이션이 `CliOverrides`에 매핑한 플래그만 설정 값에 영향을 줍니다.

애플리케이션 명령 enum에 flatten합니다.

1. 애플리케이션 자체 `Parser` 타입을 유지합니다.
2. 애플리케이션 자체 `Subcommand` enum을 유지합니다.
3. 해당 enum에 `#[command(flatten)] Config(ConfigCommand)`를 추가합니다.
4. Clap은 flatten된 `ConfigCommand` variant를 애플리케이션 자체 variant와 같은 명령 레벨로 확장합니다.
5. `Config(command)` variant를 match하고 `handle_config_command`에 전달합니다.

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

## 설정 템플릿

```bash
demo config-template --output config.example.yaml
```

출력 경로를 제공하지 않으면 명령은 현재 디렉터리에 `config.example.yaml`을 씁니다.
생성된 TOML 및 YAML 템플릿을 생성된 JSON Schema에 바인딩하려면
`--schema schemas/myapp.schema.json`를 추가하세요. 분할 YAML 템플릿은 대응하는
섹션 스키마를 바인딩합니다. 이 명령은 루트 및 섹션 스키마도 선택한 스키마 경로에
씁니다.

```bash
demo config-template --output config.example.toml --schema schemas/myapp.schema.json
```

루트 및 섹션 JSON Schema를 생성합니다.

```bash
demo config-schema --output schemas/myapp.schema.json
```

전체 런타임 설정 트리를 검증합니다.

```bash
demo config-validate
```

생성된 에디터 스키마는 분할 파일에 대해 의도적으로 필수 필드 진단을 피합니다.
`config-validate`는 include를 로드하고 기본값을 적용한 뒤 최종 `confique` 검증을
실행합니다. 검증이 성공하면 `Configuration is ok`를 출력합니다.

## 셸 완성

완성을 stdout으로 출력합니다.

```bash
demo completions zsh
```

완성을 설치합니다.

```bash
demo install-completions zsh
```

완성을 제거합니다.

```bash
demo uninstall-completions zsh
```

설치기는 Bash, Elvish, Fish, PowerShell, Zsh를 지원합니다. 사용자 홈 디렉터리
아래에 완성 파일을 쓰고, 필요한 셸에 대해서는 셸 시작 파일을 업데이트합니다.

기존 셸 시작 파일을 변경하기 전에, 예를 들어 `~/.zshrc`, `~/.bashrc`,
Elvish rc 파일 또는 PowerShell profile을 변경하기 전에, 명령은 원본 파일 옆에
백업을 씁니다.

```text
<rc-file>.backup.by.<program-name>.<timestamp>
```
