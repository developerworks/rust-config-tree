# Examples

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

이 예제들은 각자 임시 설정 파일을 만드는 작은 실행 가능한 프로그램입니다.

저장소 루트에서 실행하세요.

```bash
cargo run --example basic_loading
cargo run --example cli_overrides -- --server-port 9000
cargo run --example config_commands -- config-template --output /tmp/config.example.yaml
cargo run --example config_commands -- config-schema --output /tmp/myapp.schema.json
cargo run --example config_commands -- config-validate
cargo run --example generate_templates
cargo run --example tree_api
```

예제 내용:

- `basic_loading.rs`: 재귀 설정 트리에서 `confique` 스키마를 로드합니다.
- `cli_overrides.rs`: 애플리케이션 CLI 플래그를 가장 높은 우선순위의 Figment 프로바이더로 병합합니다.
- `config_commands.rs`: `ConfigCommand`를 애플리케이션 clap CLI에 flatten합니다.
- `generate_templates.rs`: 스키마에서 루트 및 섹션 JSON Schema와 스키마 바인딩 TOML/YAML 템플릿을 씁니다.
- `tree_api.rs`: 더 낮은 수준의 형식 독립 include 트리 API를 사용합니다.
