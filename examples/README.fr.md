# Exemples

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

Ces exemples sont de petits programmes executables qui creent leurs propres
fichiers de configuration temporaires.

Executez-les depuis la racine du depot :

```bash
cargo run --example basic_loading
cargo run --example cli_overrides -- --server-port 9000
cargo run --example config_commands -- config-template
cargo run --example config_commands -- config-schema
cargo run --example config_commands -- config-validate
cargo run --example generate_templates
cargo run --example tree_api
```

Les exemples couvrent :

- `basic_loading.rs` : charger un schema `confique` depuis un arbre de
  configuration recursif.
- `cli_overrides.rs` : fusionner des drapeaux CLI d'application comme
  fournisseur Figment de plus haute priorite.
- `config_commands.rs` : aplatir `ConfigCommand` dans une CLI clap
  d'application.
- `generate_templates.rs` : ecrire les schemas JSON racine et de section ainsi
  que des modeles TOML/YAML lies aux schemas depuis un schema.
- `tree_api.rs` : utiliser l'API d'arbre d'inclusion de plus bas niveau,
  independante du format.

