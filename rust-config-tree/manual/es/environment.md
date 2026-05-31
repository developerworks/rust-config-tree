# Variables de entorno

[English](../en/environment.html) | [中文](../zh/environment.html) | [日本語](../ja/environment.html) | [한국어](../ko/environment.html) | [Français](../fr/environment.html) | [Deutsch](../de/environment.html) | [Español](environment.html) | [Português](../pt/environment.html) | [Svenska](../sv/environment.html) | [Suomi](../fi/environment.html) | [Nederlands](../nl/environment.html)

Los nombres de variables de entorno se declaran en el esquema con `confique`:

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

`rust-config-tree` lee esos nombres desde `confique::Config::META` y construye
un proveedor Figment que mapea cada variable de entorno a su ruta exacta de
campo.

No uses mapeo de entorno de Figment basado en delimitadores para este crate:

```rust
// Do not use this pattern for rust-config-tree schemas.
Env::prefixed("APP_").split("_")
Env::prefixed("APP_").split("__")
```

`split("_")` trata los guiones bajos como separadores de claves anidadas. Eso
hace que `APP_DATABASE_POOL_SIZE` se convierta en una ruta como
`database.pool.size`, que entra en conflicto con nombres de campos Rust como
`pool_size`.

Con `ConfiqueEnvProvider`, este mapeo es explícito:

```text
APP_DATABASE_POOL_SIZE -> database.pool_size
```

Los guiones bajos simples siguen siendo parte del nombre de variable de
entorno. Figment no adivina la regla de anidamiento.

## Carga dotenv

Antes de evaluar proveedores en tiempo de ejecución, el cargador busca un
archivo `.env` caminando hacia arriba desde el directorio del archivo de
configuración raíz.

Las variables de entorno existentes del proceso se conservan. Los valores de
`.env` solo rellenan variables de entorno ausentes.

Ejemplo:

```dotenv
APP_SERVER_PORT=9000
APP_DATABASE_POOL_SIZE=64
```

Estas variables sobrescriben valores de archivos de configuración cuando el
esquema declara atributos `#[config(env = "...")]` coincidentes.

## Parseo de valores

El proveedor puente deja que Figment parsee los valores de entorno. No llama a
los hooks `parse_env` de `confique`. Mantén valores complejos en archivos de
configuración salvo que la sintaxis de valores de entorno de Figment encaje bien
con el tipo.
