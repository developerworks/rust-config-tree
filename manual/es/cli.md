# Integración CLI

[English](../en/cli.html) | [中文](../zh/cli.html) | [日本語](../ja/cli.html) | [한국어](../ko/cli.html) | [Français](../fr/cli.html) | [Deutsch](../de/cli.html) | [Español](cli.html) | [Português](../pt/cli.html) | [Svenska](../sv/cli.html) | [Suomi](../fi/cli.html) | [Nederlands](../nl/cli.html)

`ConfigCommand` proporciona subcomandos clap reutilizables:

- `config-template`
- `config-schema`
- `config-validate`
- `completions`
- `install-completions`
- `uninstall-completions`

Estos subcomandos incorporados están separados de las flags de override de
configuración específicas de la aplicación. Fusiona flags de override de
configuración como proveedores Figment en la ruta de carga en tiempo de
ejecución.

Las flags de override de configuración siguen siendo parte de la CLI de la
aplicación consumidora. Sus nombres no necesitan coincidir con rutas de
configuración con puntos. Por ejemplo, la aplicación puede parsear
`--server-port` y mapearla a la clave de configuración anidada `server.port`.
Solo las flags que la aplicación mapea dentro de `CliOverrides` afectan los
valores de configuración.

Aplánalo dentro de un enum de comandos de aplicación:

1. Mantén el tipo `Parser` propio de la aplicación.
2. Mantén el enum `Subcommand` propio de la aplicación.
3. Añade `#[command(flatten)] Config(ConfigCommand)` a ese enum.
4. Clap expande las variantes aplanadas de `ConfigCommand` en el mismo nivel de
   comandos que las variantes propias de la aplicación.
5. Haz match de la variante `Config(command)` y pásala a
   `handle_config_command`.

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

## Plantillas de configuración

```bash
demo config-template --output app_config.example.yaml
```

El comando escribe plantillas bajo `config/<root_config_name>/`. Si `--output`
recibe una ruta, solo se usa el nombre de archivo. Si no se proporciona un
nombre de archivo de salida, el comando escribe
`config/<root_config_name>/<root_config_name>.example.yaml`. Añade
`--schema schemas/myapp.schema.json` para enlazar plantillas TOML y YAML
generadas a JSON Schemas generados. Las plantillas YAML divididas enlazan el
esquema de sección correspondiente. El comando también escribe los esquemas
raíz y de sección en la ruta de esquema seleccionada.

```bash
demo config-template --output app_config.example.toml --schema schemas/myapp.schema.json
```

Genera JSON Schemas raíz y de sección:

```bash
demo config-schema --output schemas/myapp.schema.json
```

Sin `--output`, `config-schema` escribe el esquema raíz en
`config/<root_config_name>/<root_config_name>.schema.json`.

Valida el árbol completo de configuración en tiempo de ejecución:

```bash
demo config-validate
```

Los esquemas de editor generados evitan deliberadamente diagnósticos de campos
obligatorios para archivos divididos. `config-validate` carga includes, aplica
valores por defecto y ejecuta la validación final de `confique`. Imprime
`Configuration is ok` cuando la validación tiene éxito.

## Shell completions

Imprime completions a stdout:

```bash
demo completions zsh
```

Instala completions:

```bash
demo install-completions zsh
```

Desinstala completions:

```bash
demo uninstall-completions zsh
```

El instalador admite Bash, Elvish, Fish, PowerShell y Zsh. Escribe el archivo
de completion bajo el directorio home del usuario y actualiza el archivo de
inicio del shell para shells que lo requieren.

Antes de cambiar un archivo de inicio de shell existente como `~/.zshrc`,
`~/.bashrc`, un archivo rc de Elvish o un perfil de PowerShell, el comando
escribe una copia de seguridad junto al archivo original:

```text
<rc-file>.backup.by.<program-name>.<timestamp>
```
