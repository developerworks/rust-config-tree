# 環境変数

[English](../en/environment.html) | [中文](../zh/environment.html) | [日本語](environment.html) | [한국어](../ko/environment.html) | [Français](../fr/environment.html) | [Deutsch](../de/environment.html) | [Español](../es/environment.html) | [Português](../pt/environment.html) | [Svenska](../sv/environment.html) | [Suomi](../fi/environment.html) | [Nederlands](../nl/environment.html)

environment variable name は schema 内で `confique` により宣言します。

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

`rust-config-tree` は `confique::Config::META` からこれらの名前を読み取り、
各 environment variable を正確な field path へ mapping する Figment provider
を構築します。

この crate の schema では delimiter-based Figment environment mapping を使いません。

```rust
// Do not use this pattern for rust-config-tree schemas.
Env::prefixed("APP_").split("_")
Env::prefixed("APP_").split("__")
```

`split("_")` は underscore を nested key separator として扱います。そのため
`APP_DATABASE_POOL_SIZE` は `database.pool.size` のような path になり、
`pool_size` のような Rust field name と衝突します。

`ConfiqueEnvProvider` では mapping は明示的です。

```text
APP_DATABASE_POOL_SIZE -> database.pool_size
```

single underscore は environment variable name の一部として残ります。Figment は
nesting rule を推測しません。

## Dotenv Loading

runtime provider を評価する前に、loader は root config file の directory から
上方向に `.env` file を探します。

既存の process environment variables は保持されます。`.env` の値は missing
environment variable だけを埋めます。

例:

```dotenv
APP_SERVER_PORT=9000
APP_DATABASE_POOL_SIZE=64
```

schema が matching `#[config(env = "...")]` attribute を宣言している場合、
これらの variable は config file value を上書きします。

## Parsing Values

bridge provider は Figment に environment value を parse させます。
`confique` の `parse_env` hook は呼びません。complex value は、Figment の
environment value syntax が型に十分合う場合を除き、config file に置いてください。

