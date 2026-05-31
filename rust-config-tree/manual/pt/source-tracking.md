# Rastreamento de origem

[English](../en/source-tracking.html) | [中文](../zh/source-tracking.html) | [日本語](../ja/source-tracking.html) | [한국어](../ko/source-tracking.html) | [Français](../fr/source-tracking.html) | [Deutsch](../de/source-tracking.html) | [Español](../es/source-tracking.html) | [Português](source-tracking.html) | [Svenska](../sv/source-tracking.html) | [Suomi](../fi/source-tracking.html) | [Nederlands](../nl/source-tracking.html)

Use `load_config_with_figment` para manter o grafo Figment usado pelo
carregamento em tempo de execucao:

```rust
use rust_config_tree::load_config_with_figment;

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

O valor Figment retornado pode responder perguntas de origem sobre valores em
tempo de execucao:

```rust
if let Some(metadata) = figment.find_metadata("database.pool_size") {
    let source = metadata.interpolate(
        &figment::Profile::Default,
        &["database", "pool_size"],
    );

    println!("database.pool_size came from {source}");
}
```

Para valores fornecidos por `ConfiqueEnvProvider`, a interpolacao retorna o nome
nativo da variavel de ambiente declarado no esquema:

```text
database.pool_size came from APP_DATABASE_POOL_SIZE
```

## Eventos TRACE

O carregador emite eventos de rastreamento de origem com `tracing::trace!`. Ele
faz isso apenas quando TRACE esta habilitado:

```rust
use rust_config_tree::{load_config_with_figment, trace_config_sources};

let (config, figment) = load_config_with_figment::<AppConfig>("config.yaml")?;

// If the tracing subscriber is initialized after config loading, emit the
// same source events again after installing the subscriber.
trace_config_sources::<AppConfig>(&figment);
# let _ = config;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Cada evento usa o target `rust_config_tree::config` e inclui:

- `config_key`: a chave de configuracao pontuada.
- `source`: os metadados de origem renderizados.

Valores que vieram apenas de padroes do `confique` nao tem metadados Figment em
tempo de execucao. Eles sao relatados como
`confique default or unset optional field`.

