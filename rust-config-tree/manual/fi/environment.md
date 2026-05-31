# Ymparistomuuttujat

[English](../en/environment.html) | [中文](../zh/environment.html) | [日本語](../ja/environment.html) | [한국어](../ko/environment.html) | [Français](../fr/environment.html) | [Deutsch](../de/environment.html) | [Español](../es/environment.html) | [Português](../pt/environment.html) | [Svenska](../sv/environment.html) | [Suomi](environment.html) | [Nederlands](../nl/environment.html)

Ymparistomuuttujien nimet maaritellaan skeemassa `confique`-attribuuteilla:

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

`rust-config-tree` lukee nama nimet `confique::Config::META`-metadatasta ja rakentaa Figment-providerin, joka mapittaa jokaisen ymparistomuuttujan tarkkaan kenttapolkuunsa.

Ala kayta erotinmerkkeihin perustuvaa Figmentin ymparistomappausta taman craten kanssa:

```rust
// Do not use this pattern for rust-config-tree schemas.
Env::prefixed("APP_").split("_")
Env::prefixed("APP_").split("__")
```

`split("_")` tulkitsee alaviivat sisakkaisten avainten erottimiksi. Silloin `APP_DATABASE_POOL_SIZE` muuttuu poluksi kuten `database.pool.size`, joka on ristiriidassa Rust-kenttanimien kuten `pool_size` kanssa.

`ConfiqueEnvProvider`-providerilla mappaus on eksplisiittinen:

```text
APP_DATABASE_POOL_SIZE -> database.pool_size
```

Yksittaiset alaviivat pysyvat osana ymparistomuuttujan nimea. Figment ei arvaa sisakkaisyyssaantoa.

## Dotenv-lataus

Ennen runtime-providereiden arviointia lataaja etsii `.env`-tiedostoa kulkemalla ylospain juurikonfiguraatiotiedoston hakemistosta.

Prosessin olemassa olevat ymparistomuuttujat sailyvat. `.env`-arvot tayttavat vain puuttuvat ymparistomuuttujat.

Esimerkki:

```dotenv
APP_SERVER_PORT=9000
APP_DATABASE_POOL_SIZE=64
```

Nama muuttujat ohittavat konfiguraatiotiedoston arvot, kun skeema maarittelee vastaavat `#[config(env = "...")]`-attribuutit.

## Arvojen jasantaminen

Siltaprovider antaa Figmentin jasantaa ymparistoarvot. Se ei kutsu `confique`-kirjaston `parse_env`-hookeja. Pida monimutkaiset arvot konfiguraatiotiedostoissa, ellei Figmentin ymparistoarvosyntaksi sovi tyypille hyvin.
