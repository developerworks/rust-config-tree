# Variaveis de ambiente

[English](../en/environment.html) | [中文](../zh/environment.html) | [日本語](../ja/environment.html) | [한국어](../ko/environment.html) | [Français](../fr/environment.html) | [Deutsch](../de/environment.html) | [Español](../es/environment.html) | [Português](environment.html) | [Svenska](../sv/environment.html) | [Suomi](../fi/environment.html) | [Nederlands](../nl/environment.html)

Nomes de variaveis de ambiente sao declarados no esquema com `confique`:

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

`rust-config-tree` le esses nomes de `confique::Config::META` e constroi um
provedor Figment que mapeia cada variavel de ambiente para seu caminho exato de
campo.

Nao use mapeamento de ambiente Figment baseado em delimitador para este crate:

```rust
// Do not use this pattern for rust-config-tree schemas.
Env::prefixed("APP_").split("_")
Env::prefixed("APP_").split("__")
```

`split("_")` trata underscores como separadores de chaves aninhadas. Isso faz
`APP_DATABASE_POOL_SIZE` virar um caminho como `database.pool.size`, que entra em
conflito com nomes de campos Rust como `pool_size`.

Com `ConfiqueEnvProvider`, esse mapeamento e explicito:

```text
APP_DATABASE_POOL_SIZE -> database.pool_size
```

Underscores simples continuam fazendo parte do nome da variavel de ambiente.
Figment nao tenta adivinhar a regra de aninhamento.

## Carregamento dotenv

Antes que provedores de tempo de execucao sejam avaliados, o carregador procura
um arquivo `.env` subindo a partir do diretorio do arquivo de configuracao raiz.

Variaveis de ambiente existentes no processo sao preservadas. Valores de `.env`
apenas preenchem variaveis de ambiente ausentes.

Exemplo:

```dotenv
APP_SERVER_PORT=9000
APP_DATABASE_POOL_SIZE=64
```

Essas variaveis sobrescrevem valores de arquivos de configuracao quando o
esquema declara atributos `#[config(env = "...")]` correspondentes.

## Analise de valores

O provedor de ponte deixa o Figment analisar valores de ambiente. Ele nao chama
os hooks `parse_env` do `confique`. Mantenha valores complexos em arquivos de
configuracao a menos que a sintaxe de valores de ambiente do Figment seja uma
boa escolha para o tipo.

