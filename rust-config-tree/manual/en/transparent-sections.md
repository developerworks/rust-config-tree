# Transparent Array Sections

[English](transparent-sections.html) | [中文](../zh/transparent-sections.html)

## Overview

A **transparent array section** lets a split config section appear as a YAML
array while `confique` stores the data in an inner `items` field. The loader,
template generator, and section schema all treat the section as an array rather
than an `{ items: [...] }` object.

In short: write `children: [...]` in a single file, or write only `[...]` in a
split file without an `items:` wrapper.

Use this when list-shaped configuration (worker declarations, route tables,
plugin lists) should live in its own split file while keeping that file as a
plain array body.

## Schema Markers

Transparent array sections require both `x-tree-split` and
`x-tree-transparent-array`:

```rust
use rust_config_tree::transparent_array_section;

transparent_array_section! {
    /// Child declarations stored as a transparent array section.
    pub struct ChildrenSection {
        #[config(default = [{ "name": "worker" }])]
        pub items: Vec<ChildDeclaration>,
    }
}

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(nested)]
    #[schemars(extend(
        "x-tree-split" = true,
        "x-tree-transparent-array" = true
    ))]
    children: ChildrenSection,
}
```

| Extension                       | Purpose                                                             |
| ------------------------------- | ------------------------------------------------------------------- |
| `x-tree-split`                  | Emit a separate `children.yaml` template and `children.schema.json` |
| `x-tree-transparent-array`      | Treat the section as a YAML array at runtime and in templates       |
| `x-tree-inner-field` (optional) | Override the inner `confique` field name; default is `"items"`      |

## Rust Type Options

### `transparent_array_section!` macro

The macro generates `len`, `is_empty`, `as_slice`, and `From<SectionName> for Vec<T>`. Each section struct can declare its own
`#[config(default = ...)]` template sample.

### Generic `ArraySection<T>`

Skip the macro and use `ArraySection<T>` directly:

```rust
use rust_config_tree::ArraySection;

#[derive(Debug, Config, JsonSchema)]
struct ChildrenSection {
    #[config(default = [{ "name": "worker" }])]
    items: Vec<ChildDeclaration>,
}
```

`ArraySection<T>` supports only one `#[config(default)]`. When multiple sections
need different template defaults, define a separate struct per section or use
the macro.

## YAML Shapes

### Single file

Write the array directly in the root config:

```yaml
children:
  - name: api
  - name: worker
```

### Split file

Reference the split file from the root config:

```yaml
include:
  - children.yaml
mode: demo
```

`children.yaml` contains **only the array body**. Do not add a section root key
or an `items:` wrapper:

```yaml
- name: api
- name: worker
```

### Shapes accepted by the loader

`load_config` accepts all three forms:

1. Transparent array: `children: [...]`
2. Explicit inner field: `children:\n  items: [...]`
3. Body-only split file: `children.yaml` contains `[...]`

The loader merges split files into `children: { items: [...] }` before
`confique` deserializes the final config.

## Runtime defaults vs template defaults

Template generation uses `#[config(default = ...)]` to write sample entries
(for example a `worker` row).

When the transparent section is **omitted entirely** at runtime, the library
injects `{ items: [] }` through `TransparentSectionTracker` so template defaults
do not leak into runtime as phantom entries.

Therefore:

- **Template defaults** guide authors and appear in `generate-template` output.
- **Runtime defaults** yield an empty array when the section is omitted, not the
  template sample worker.

## Templates and schemas

Generated `children.schema.json` uses a top-level `array` type, so IDEs complete
array items directly while editing `children.yaml`.

Template generation emits block YAML array bodies:

- No `children:` root key
- No `items:` wrapper
- No flow-style `[{ ... }]`

The root template `config.example.yaml` includes `include: [children.yaml]`.

## Accessing data

```rust
config.children.len();
config.children.is_empty();
config.children.as_slice();
let vec: Vec<ChildDeclaration> = config.children.into();
```

Both the macro and `ArraySection` expose this API surface.

## Complete example

The repository ships a runnable demo:

```bash
cargo run --example transparent_array_section
```

The example:

1. Writes split config files into a temporary directory
2. Generates the section schema
3. Calls `load_config` and verifies transparent array loading

For a downstream project, see
[rust-supervisor split-config](https://github.com/developerworks/rust-supervisor/blob/main/manual/en/split-config.md)
for real `groups` and `children` usage.

## Related pages

- [Configuration Schema](schema.md) — nested sections and split markers
- [Template Generation](templates.md) — split section template output rules
- [Runtime Loading](runtime-loading.md) — loading steps and merge precedence
- [IDE Completions](ide-completions.md) — section schema bindings
