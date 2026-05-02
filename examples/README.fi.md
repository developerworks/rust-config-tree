# Esimerkit

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

Nama esimerkit ovat pienia ajettavia ohjelmia, jotka luovat omat valiaikaiset konfiguraatiotiedostonsa.

Aja ne repositorion juuresta:

```bash
cargo run --example basic_loading
cargo run --example cli_overrides -- --server-port 9000
cargo run --example config_commands -- config-template --output /tmp/config.example.yaml
cargo run --example config_commands -- config-schema --output /tmp/myapp.schema.json
cargo run --example config_commands -- config-validate
cargo run --example generate_templates
cargo run --example tree_api
```

Esimerkit kattavat:

- `basic_loading.rs`: lataa `confique`-skeema rekursiivisesta konfiguraatiopuusta.
- `cli_overrides.rs`: yhdista sovelluksen CLI-liput korkeimman prioriteetin Figment-provideriksi.
- `config_commands.rs`: litista `ConfigCommand` sovelluksen clap-CLI:hin.
- `generate_templates.rs`: kirjoita juuri- ja osio-JSON Schema -skeemat seka skeemaan sidotut TOML/YAML-mallit skeemasta.
- `tree_api.rs`: kayta alemman tason, formaattiriippumatonta include-puu-APIa.
