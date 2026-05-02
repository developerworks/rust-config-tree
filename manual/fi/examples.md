# Esimerkit

[English](../en/examples.html) | [中文](../zh/examples.html) | [日本語](../ja/examples.html) | [한국어](../ko/examples.html) | [Français](../fr/examples.html) | [Deutsch](../de/examples.html) | [Español](../es/examples.html) | [Português](../pt/examples.html) | [Svenska](../sv/examples.html) | [Suomi](examples.html) | [Nederlands](../nl/examples.html)

Repository sisaltaa ajettavat esimerkit konfiguraatiopuiden lataukseen, CLI-ohituksiin, sisaanrakennettuihin konfiguraatiokomentoihin, mallien luontiin ja alemman tason puu-APIin.

Lue repositorion esimerkki-indeksi:

- [examples/README.md](https://github.com/developerworks/rust-config-tree/blob/main/examples/README.md)

Aja esimerkit repositorion juuresta:

```bash
cargo run --example basic_loading
cargo run --example cli_overrides -- --server-port 9000
cargo run --example config_commands -- config-template --output /tmp/config.example.yaml
cargo run --example config_commands -- config-schema --output /tmp/myapp.schema.json
cargo run --example config_commands -- config-validate
cargo run --example generate_templates
cargo run --example tree_api
```
