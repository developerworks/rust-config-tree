# 예제

[English](../en/examples.html) | [中文](../zh/examples.html) | [日本語](../ja/examples.html) | [한국어](examples.html) | [Français](../fr/examples.html) | [Deutsch](../de/examples.html) | [Español](../es/examples.html) | [Português](../pt/examples.html) | [Svenska](../sv/examples.html) | [Suomi](../fi/examples.html) | [Nederlands](../nl/examples.html)

저장소에는 설정 트리 로딩, CLI override, 내장 설정 명령, 템플릿 생성, 낮은 수준의
트리 API를 위한 실행 가능한 예제가 포함되어 있습니다.

저장소 예제 색인을 읽어보세요.

- [examples/README.md](https://github.com/developerworks/rust-config-tree/blob/main/examples/README.md)

저장소 루트에서 예제를 실행합니다.

```bash
cargo run --example basic_loading
cargo run --example cli_overrides -- --server-port 9000
cargo run --example config_commands -- config-template --output /tmp/config.example.yaml
cargo run --example config_commands -- config-schema --output /tmp/myapp.schema.json
cargo run --example config_commands -- config-validate
cargo run --example generate_templates
cargo run --example tree_api
```
