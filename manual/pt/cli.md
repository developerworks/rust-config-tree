# Integracao de CLI

[English](../en/cli.html) | [中文](../zh/cli.html) | [日本語](../ja/cli.html) | [한국어](../ko/cli.html) | [Français](../fr/cli.html) | [Deutsch](../de/cli.html) | [Español](../es/cli.html) | [Português](cli.html) | [Svenska](../sv/cli.html) | [Suomi](../fi/cli.html) | [Nederlands](../nl/cli.html)

`ConfigCommand` fornece subcomandos clap reutilizaveis:

- `config-template`
- `config-schema`
- `config-validate`
- `completions`
- `install-completions`
- `uninstall-completions`

Esses subcomandos embutidos sao separados das flags de sobrescrita de
configuracao especificas da aplicacao. Mescle flags de sobrescrita de
configuracao como provedores Figment no caminho de carregamento em tempo de
execucao.

Flags de sobrescrita de configuracao continuam fazendo parte da CLI da
aplicacao consumidora. Seus nomes nao precisam corresponder a caminhos de
configuracao pontuados. Por exemplo, a aplicacao pode analisar `--server-port` e
mapeia-lo para a chave aninhada `server.port`. Somente flags que a aplicacao
mapeia para `CliOverrides` afetam valores de configuracao.

Achate-o em um enum de comandos da aplicacao:

1. Mantenha o tipo `Parser` proprio da aplicacao.
2. Mantenha o enum `Subcommand` proprio da aplicacao.
3. Adicione `#[command(flatten)] Config(ConfigCommand)` a esse enum.
4. Clap expande as variantes achatadas de `ConfigCommand` para o mesmo nivel de
   comando das variantes proprias da aplicacao.
5. Faca match da variante `Config(command)` e passe-a para
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

## Modelos de configuracao

```bash
demo config-template
```

O comando grava modelos em `config/<root_config_name>/`. Se `--output` receber
um caminho, somente o nome do arquivo e usado. Se nenhum nome de arquivo de
saida for fornecido, o comando grava
`config/<root_config_name>/<root_config_name>.example.yaml`. Adicione
`--schema schemas/myapp.schema.json` para vincular modelos TOML, YAML, JSON e
JSON5 gerados a JSON Schemas gerados. Modelos YAML divididos vinculam o esquema
de secao correspondente. Modelos JSON e JSON5 recebem um campo `$schema`
reconhecido pelo VS Code. O comando tambem grava o esquema raiz e esquemas de
secao no caminho de esquema selecionado.

```bash
demo config-template --output app_config.example.toml --schema schemas/myapp.schema.json
```

Gere JSON Schemas raiz e de secao:

```bash
demo config-schema
```

Sem `--output`, `config-schema` grava o esquema raiz em
`config/<root_config_name>/<root_config_name>.schema.json`.

Valide a arvore completa de configuracao em tempo de execucao:

```bash
demo config-validate
```

Esquemas de editor gerados evitam intencionalmente diagnosticos de campos
obrigatorios para arquivos divididos. `config-validate` carrega includes, aplica
padroes e executa a validacao final do `confique`, incluindo validadores
declarados com `#[config(validate = Self::validate)]`. Os `*.schema.json`
gerados continuam sendo para completamento de IDE e verificacoes basicas do
editor, nao para legalidade de valores de campo. Ele imprime `Configuration is
ok` quando a validacao tem sucesso.

## Shell completions

Imprima completions em stdout:

```bash
demo completions zsh
```

Instale completions:

```bash
demo install-completions zsh
```

Desinstale completions:

```bash
demo uninstall-completions zsh
```

O instalador suporta Bash, Elvish, Fish, PowerShell e Zsh. Ele grava o arquivo
de completion sob o diretorio home do usuario e atualiza o arquivo de
inicializacao do shell para shells que exigem isso.

Antes de alterar um arquivo de inicializacao de shell existente, como
`~/.zshrc`, `~/.bashrc`, um arquivo rc do Elvish ou um perfil do PowerShell, o
comando grava um backup ao lado do arquivo original:

```text
<rc-file>.backup.by.<program-name>.<timestamp>
```
