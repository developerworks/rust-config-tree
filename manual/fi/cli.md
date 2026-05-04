# CLI-integraatio

[English](../en/cli.html) | [中文](../zh/cli.html) | [日本語](../ja/cli.html) | [한국어](../ko/cli.html) | [Français](../fr/cli.html) | [Deutsch](../de/cli.html) | [Español](../es/cli.html) | [Português](../pt/cli.html) | [Svenska](../sv/cli.html) | [Suomi](cli.html) | [Nederlands](../nl/cli.html)

`ConfigCommand` tarjoaa uudelleenkaytettavat clap-alikomennot:

- `config-template`
- `config-schema`
- `config-validate`
- `completions`
- `install-completions`
- `uninstall-completions`

Nama sisaanrakennetut alikomennot ovat erillaan sovelluskohtaisista konfiguraation ohituslipuista. Yhdista konfiguraation ohitusliput Figment-providereina runtime-latauspolussa.

Konfiguraation ohitusliput pysyvat kayttavan sovelluksen CLI:n osana. Niiden nimien ei tarvitse vastata pisteellisia konfiguraatiopolkuja. Sovellus voi esimerkiksi jasantaa `--server-port`-lipun ja mapittaa sen sisakkaiseen `server.port`-konfiguraatioavaimeen. Vain liput, jotka sovellus mapittaa `CliOverrides`-rakenteeseen, vaikuttavat konfiguraatioarvoihin.

Litista se sovelluksen komentoenumiin:

1. Sailyta sovelluksen oma `Parser`-tyyppi.
2. Sailyta sovelluksen oma `Subcommand`-enum.
3. Lisaa kyseiseen enumiin `#[command(flatten)] Config(ConfigCommand)`.
4. Clap laajentaa litistetyt `ConfigCommand`-variantit samalle komentotasolle kuin sovelluksen omat variantit.
5. Tasmaa `Config(command)`-variantti ja valita se `handle_config_command`-funktiolle.

```rust
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use confique::Config;
use schemars::JsonSchema;
use rust_config_tree::{ConfigCommand, ConfigSchema, handle_config_command, load_config};

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,
}

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}

#[derive(Debug, Parser)]
#[command(name = "demo")]
struct Cli {
    #[arg(long, default_value = "config.yaml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Run,

    #[command(flatten)]
    Config(ConfigCommand),
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Run => {
            let config = load_config::<AppConfig>(&cli.config)?;
            println!("{config:#?}");
        }
        Command::Config(command) => {
            handle_config_command::<Cli, AppConfig>(command, &cli.config)?;
        }
    }

    Ok(())
}
```

## Konfiguraatiomallit

```bash
demo config-template --output app_config.example.yaml
```

Komento kirjoittaa mallit hakemistoon `config/<root_config_name>/`. Jos `--output` saa polun, vain tiedostonimi kaytetaan. Jos tulostetiedoston nimea ei anneta, komento kirjoittaa `config/<root_config_name>/<root_config_name>.example.yaml`. Lisaa `--schema schemas/myapp.schema.json`, jotta luodut TOML- ja YAML-mallit sidotaan luotuihin JSON Schema -skeemoihin. Jaetut YAML-mallit sitovat vastaavan osioskeeman. Komento kirjoittaa myos juuri- ja osioskeemat valittuun skeemapolkuun.

```bash
demo config-template --output app_config.example.toml --schema schemas/myapp.schema.json
```

Luo juuri- ja osio-JSON Schema -skeemat:

```bash
demo config-schema --output schemas/myapp.schema.json
```

Ilman `--output`-arvoa `config-schema` kirjoittaa juuriskeeman tiedostoon `config/<root_config_name>/<root_config_name>.schema.json`.

Validoi koko runtime-konfiguraatiopuu:

```bash
demo config-validate
```

Luodut editoriskeemat valttavat tarkoituksella required-kenttien diagnostiikkaa jaetuille tiedostoille. `config-validate` lataa includet, kayttaa oletusarvot ja ajaa lopullisen `confique`-validoinnin, mukaan lukien `#[config(validate = Self::validate)]`-attribuutilla maaritellyt validaattorit. Luodut `*.schema.json`-tiedostot ovat IDE-taydennysta ja editorin perustarkistuksia varten, eivat kentta-arvon kelvollisuuden arviointiin. Se tulostaa `Configuration is ok`, kun validointi onnistuu.

## Shell-taydennykset

Tulosta taydennykset stdoutiin:

```bash
demo completions zsh
```

Asenna taydennykset:

```bash
demo install-completions zsh
```

Poista taydennykset:

```bash
demo uninstall-completions zsh
```

Asennin tukee Bashia, Elvishia, Fishiä, PowerShellia ja Zsh:ta. Se kirjoittaa taydennystiedoston kayttajan kotihakemiston alle ja paivittaa shellin kaynnistystiedoston niille shelleille, jotka sita vaativat.

Ennen olemassa olevan shellin kaynnistystiedoston, kuten `~/.zshrc`,
`~/.bashrc`, Elvish rc -tiedoston tai PowerShell-profiilin muuttamista komento
kirjoittaa varmuuskopion alkuperaisen tiedoston viereen:

```text
<rc-file>.backup.by.<program-name>.<timestamp>
```
