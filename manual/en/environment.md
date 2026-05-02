# Environment Variables

[English](environment.md) | [中文](../zh/environment.md)

Environment variable names are declared in the schema with `confique`:

```rust
#[derive(Debug, Config)]
struct DatabaseConfig {
    #[config(env = "APP_DATABASE_URL")]
    url: String,

    #[config(default = 16)]
    #[config(env = "APP_DATABASE_POOL_SIZE")]
    pool_size: u32,
}
```

`rust-config-tree` reads those names from `confique::Config::META` and builds a
Figment provider that maps each environment variable to its exact field path.

Do not use delimiter-based Figment environment mapping for this crate:

```rust
// Do not use this pattern for rust-config-tree schemas.
Env::prefixed("APP_").split("_")
Env::prefixed("APP_").split("__")
```

`split("_")` treats underscores as nested key separators. That makes
`APP_DATABASE_POOL_SIZE` become a path like `database.pool.size`, which conflicts
with Rust field names such as `pool_size`.

With `ConfiqueEnvProvider`, this mapping is explicit:

```text
APP_DATABASE_POOL_SIZE -> database.pool_size
```

Single underscores remain part of the environment variable name. Figment does
not guess the nesting rule.

## Dotenv Loading

Before runtime providers are evaluated, the loader searches for a `.env` file by
walking upward from the root config file's directory.

Existing process environment variables are preserved. Values from `.env` only
fill missing environment variables.

Example:

```dotenv
APP_SERVER_PORT=9000
APP_DATABASE_POOL_SIZE=64
```

These variables override config file values when the schema declares matching
`#[config(env = "...")]` attributes.

## Parsing Values

The bridge provider lets Figment parse environment values. It does not call
`confique`'s `parse_env` hooks. Keep complex values in config files unless the
Figment environment value syntax is a good fit for the type.
