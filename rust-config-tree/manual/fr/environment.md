# Variables d'environnement

[English](../en/environment.html) | [中文](../zh/environment.html) | [日本語](../ja/environment.html) | [한국어](../ko/environment.html) | [Français](environment.html) | [Deutsch](../de/environment.html) | [Español](../es/environment.html) | [Português](../pt/environment.html) | [Svenska](../sv/environment.html) | [Suomi](../fi/environment.html) | [Nederlands](../nl/environment.html)

Les noms de variables d'environnement sont declares dans le schema avec
`confique` :

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

`rust-config-tree` lit ces noms depuis `confique::Config::META` et construit un
fournisseur Figment qui associe chaque variable d'environnement a son chemin de
champ exact.

N'utilisez pas le mapping d'environnement Figment base sur des delimiteurs avec
cette crate :

```rust
// Do not use this pattern for rust-config-tree schemas.
Env::prefixed("APP_").split("_")
Env::prefixed("APP_").split("__")
```

`split("_")` traite les underscores comme des separateurs de cles imbriquees.
Cela transforme `APP_DATABASE_POOL_SIZE` en un chemin comme
`database.pool.size`, ce qui entre en conflit avec les noms de champs Rust comme
`pool_size`.

Avec `ConfiqueEnvProvider`, ce mapping est explicite :

```text
APP_DATABASE_POOL_SIZE -> database.pool_size
```

Les underscores simples restent une partie du nom de variable d'environnement.
Figment ne devine pas la regle d'imbrication.

## Chargement dotenv

Avant l'evaluation des fournisseurs d'execution, le chargeur cherche un fichier
`.env` en remontant depuis le repertoire du fichier de configuration racine.

Les variables d'environnement deja presentes dans le processus sont conservees.
Les valeurs de `.env` ne remplissent que les variables d'environnement
manquantes.

Exemple :

```dotenv
APP_SERVER_PORT=9000
APP_DATABASE_POOL_SIZE=64
```

Ces variables remplacent les valeurs des fichiers de configuration lorsque le
schema declare des attributs `#[config(env = "...")]` correspondants.

## Analyse des valeurs

Le fournisseur passerelle laisse Figment analyser les valeurs d'environnement.
Il n'appelle pas les hooks `parse_env` de `confique`. Gardez les valeurs
complexes dans les fichiers de configuration sauf si la syntaxe de valeur
d'environnement Figment convient au type.

