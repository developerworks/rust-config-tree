# Examples

These examples are small runnable programs that create their own temporary
config files.

Run them from the repository root:

```bash
cargo run --example basic_loading
cargo run --example cli_overrides -- --server-port 9000
cargo run --example config_commands -- config-template --output /tmp/config.example.yaml
cargo run --example config_commands -- config-schema --output /tmp/myapp.schema.json
cargo run --example config_commands -- config-validate
cargo run --example generate_templates
cargo run --example tree_api
```

The examples cover:

- `basic_loading.rs`: load a `confique` schema from a recursive config tree.
- `cli_overrides.rs`: merge application CLI flags as the highest-priority
  Figment provider.
- `config_commands.rs`: flatten `ConfigCommand` into an application clap CLI.
- `generate_templates.rs`: write root and section JSON Schemas plus
  schema-bound TOML/YAML templates from a schema.
- `tree_api.rs`: use the lower-level, format-agnostic include tree API.
