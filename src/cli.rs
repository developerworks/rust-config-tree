//! Clap subcommand integration and shell completion installation helpers.
//!
//! This module exposes reusable commands for generating config templates,
//! printing shell completions, and installing or uninstalling completions in
//! common shell startup locations.

use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use clap::{CommandFactory, Subcommand};
use clap_complete::{
    Generator,
    aot::{Shell, generate, generate_to},
};
use schemars::JsonSchema;

use crate::{
    ConfigResult, ConfigSchema,
    config::{
        default_config_schema_output, load_config, resolve_config_template_output,
        write_config_schemas, write_config_templates_with_schema,
    },
};

/// Built-in clap subcommands for config templates and shell completions.
#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// Generate an example config template.
    ///
    /// The output format is inferred from the extension; unknown or missing extensions use YAML.
    ConfigTemplate {
        /// Template file name. Defaults to `config/<root-config-name>/<root-config-name>.example.yaml`.
        #[arg(long)]
        output: Option<PathBuf>,

        /// Root JSON Schema path to write and bind from generated templates.
        #[arg(long)]
        schema: Option<PathBuf>,
    },

    /// Generate JSON Schema files for editor completion and validation.
    #[command(name = "config-schema")]
    JsonSchema {
        /// Root schema output path. Defaults to `config/<root-config-name>/<root-config-name>.schema.json`.
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Validate the full runtime config tree.
    #[command(name = "config-validate")]
    ConfigValidate,

    /// Generate shell completions.
    Completions {
        /// Shell to generate completions for.
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Install shell completions and configure the shell startup file when needed.
    InstallCompletions {
        /// Shell to install completions for.
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Uninstall shell completions and remove managed startup-file blocks.
    UninstallCompletions {
        /// Shell to uninstall completions for.
        #[arg(value_enum)]
        shell: Shell,
    },
}

/// Handles a built-in config subcommand for a consumer CLI.
///
/// `C` is the clap parser type used to generate completion metadata. `S` is the
/// application config schema used for template and JSON Schema generation.
///
/// # Type Parameters
///
/// - `C`: The consumer CLI parser type that implements [`CommandFactory`].
/// - `S`: The consumer config schema used when rendering config templates and
///   JSON Schema files.
///
/// # Arguments
///
/// - `command`: Built-in subcommand selected by the consumer CLI.
/// - `config_path`: Root config path used as the template source when handling
///   `config-template`.
///
/// # Returns
///
/// Returns `Ok(())` after the selected subcommand completes.
///
/// # Examples
///
/// ```no_run
/// use clap::{Parser, Subcommand};
/// use confique::Config;
/// use rust_config_tree::{ConfigCommand, ConfigSchema, handle_config_command};
/// use schemars::JsonSchema;
///
/// #[derive(Parser)]
/// struct Cli {
///     #[command(subcommand)]
///     command: Command,
/// }
///
/// #[derive(Subcommand)]
/// enum Command {
///     #[command(flatten)]
///     Config(ConfigCommand),
/// }
///
/// #[derive(Config, JsonSchema)]
/// struct AppConfig {
///     #[config(default = [])]
///     include: Vec<std::path::PathBuf>,
/// }
///
/// impl ConfigSchema for AppConfig {
///     fn include_paths(layer: &<Self as Config>::Layer) -> Vec<std::path::PathBuf> {
///         layer.include.clone().unwrap_or_default()
///     }
/// }
///
/// handle_config_command::<Cli, AppConfig>(
///     ConfigCommand::ConfigValidate,
///     std::path::Path::new("config.yaml"),
/// )?;
/// # Ok::<(), rust_config_tree::ConfigError>(())
/// ```
pub fn handle_config_command<C, S>(command: ConfigCommand, config_path: &Path) -> ConfigResult<()>
where
    C: CommandFactory,
    S: ConfigSchema + JsonSchema,
{
    match command {
        ConfigCommand::ConfigTemplate { output, schema } => {
            let output = resolve_config_template_output::<S>(output)?;
            let schema = schema.unwrap_or_else(default_config_schema_output::<S>);
            write_config_schemas::<S>(&schema)?;
            write_config_templates_with_schema::<S>(config_path, output, schema)
        }
        ConfigCommand::JsonSchema { output } => {
            write_config_schemas::<S>(output.unwrap_or_else(default_config_schema_output::<S>))
        }
        ConfigCommand::ConfigValidate => {
            load_config::<S>(config_path)?;
            println!("Configuration is ok");
            Ok(())
        }
        ConfigCommand::Completions { shell } => {
            print_shell_completion::<C>(shell);
            Ok(())
        }
        ConfigCommand::InstallCompletions { shell } => install_shell_completion::<C>(shell),
        ConfigCommand::UninstallCompletions { shell } => uninstall_shell_completion::<C>(shell),
    }
}

/// Writes shell completion output to stdout.
///
/// # Type Parameters
///
/// - `C`: The consumer CLI parser type used to build the clap command.
///
/// # Arguments
///
/// - `shell`: Shell whose completion script should be generated.
///
/// # Returns
///
/// This function writes to stdout and returns no value.
///
/// # Examples
///
/// ```no_run
/// use clap::Parser;
/// use clap_complete::aot::Shell;
/// use rust_config_tree::print_shell_completion;
///
/// #[derive(Parser)]
/// #[command(name = "myapp")]
/// struct Cli {}
///
/// print_shell_completion::<Cli>(Shell::Bash);
/// ```
pub fn print_shell_completion<C>(shell: Shell)
where
    C: CommandFactory,
{
    let mut cmd = C::command();
    let bin_name = cmd.get_name().to_string();
    generate(shell, &mut cmd, bin_name, &mut io::stdout());
}

/// Generates shell completion files and updates shell startup files when needed.
///
/// # Type Parameters
///
/// - `C`: The consumer CLI parser type used to build the clap command.
///
/// # Arguments
///
/// - `shell`: Shell whose completion file should be installed.
///
/// # Returns
///
/// Returns `Ok(())` after the completion file is generated and any required
/// startup file has been updated.
///
/// # Examples
///
/// ```no_run
/// use clap::Parser;
/// use clap_complete::aot::Shell;
/// use rust_config_tree::install_shell_completion;
///
/// #[derive(Parser)]
/// #[command(name = "myapp")]
/// struct Cli {}
///
/// install_shell_completion::<Cli>(Shell::Zsh)?;
/// # Ok::<(), rust_config_tree::ConfigError>(())
/// ```
pub fn install_shell_completion<C>(shell: Shell) -> ConfigResult<()>
where
    C: CommandFactory,
{
    let home_dir = home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "cannot find home directory"))?;
    let target = ShellInstallTarget::new(shell, &home_dir)?;

    fs::create_dir_all(&target.completion_dir)?;

    let mut cmd = C::command();
    let bin_name = cmd.get_name().to_string();
    let generated_path = generate_to(shell, &mut cmd, bin_name.clone(), &target.completion_dir)?;

    if let Some(ref rc_path) = target.rc_path {
        let block_body = target
            .rc_block_body(&generated_path, &target.completion_dir)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "completion install path is not valid UTF-8",
                )
            })?;
        upsert_managed_block_with_backup_name(
            &target.managed_block_name(&bin_name),
            shell,
            rc_path,
            &block_body,
            &bin_name,
        )?;
        println!("{shell} rc configured: {}", rc_path.display());
    }

    println!("{shell} completion generated: {}", generated_path.display());
    println!("restart {shell} or open a new shell session");

    Ok(())
}

/// Removes shell completion files and managed shell startup-file blocks.
///
/// # Type Parameters
///
/// - `C`: The consumer CLI parser type used to build the clap command.
///
/// # Arguments
///
/// - `shell`: Shell whose completion file should be removed.
///
/// # Returns
///
/// Returns `Ok(())` after the completion file is removed and any managed
/// startup-file block has been removed. Existing startup files are backed up
/// before being modified.
///
/// # Examples
///
/// ```no_run
/// use clap::Parser;
/// use clap_complete::aot::Shell;
/// use rust_config_tree::uninstall_shell_completion;
///
/// #[derive(Parser)]
/// #[command(name = "myapp")]
/// struct Cli {}
///
/// uninstall_shell_completion::<Cli>(Shell::Zsh)?;
/// # Ok::<(), rust_config_tree::ConfigError>(())
/// ```
pub fn uninstall_shell_completion<C>(shell: Shell) -> ConfigResult<()>
where
    C: CommandFactory,
{
    let home_dir = home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "cannot find home directory"))?;
    let target = ShellInstallTarget::new(shell, &home_dir)?;

    let cmd = C::command();
    let bin_name = cmd.get_name().to_string();
    let completion_path = target.completion_file_path(&bin_name);

    remove_completion_file(&completion_path)?;

    if let Some(ref rc_path) = target.rc_path {
        let removed_rc = if shell == Shell::Zsh {
            if completion_dir_is_empty(&target.completion_dir)? {
                remove_managed_block_with_backup_name(
                    &target.managed_block_name(&bin_name),
                    shell,
                    rc_path,
                    &bin_name,
                )?
            } else {
                false
            }
        } else {
            remove_managed_block_with_backup_name(
                &target.managed_block_name(&bin_name),
                shell,
                rc_path,
                &bin_name,
            )?
        };

        if removed_rc {
            println!("{shell} rc unconfigured: {}", rc_path.display());
        }
    }

    println!("{shell} completion removed: {}", completion_path.display());
    println!("restart {shell} or open a new shell session");

    Ok(())
}

/// Resolves the current user's home directory from environment variables.
///
/// # Arguments
///
/// This function has no arguments.
///
/// # Returns
///
/// Returns the home directory when `HOME` or `USERPROFILE` is set.
///
/// # Examples
///
/// ```no_run
/// // Internal helper; use `install_shell_completion` to resolve install paths.
/// ```
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
}

/// Completion and startup-file paths for one shell.
///
/// The completion directory receives the generated completion file. The
/// optional startup path is updated only for shells that require explicit
/// startup configuration.
struct ShellInstallTarget {
    shell: Shell,
    completion_dir: PathBuf,
    rc_path: Option<PathBuf>,
}

/// Shell-specific completion install path construction.
impl ShellInstallTarget {
    /// Creates an install target rooted under `home_dir`.
    ///
    /// # Arguments
    ///
    /// - `shell`: Shell whose completion target should be created.
    /// - `home_dir`: Home directory used as the base for completion and startup
    ///   file paths.
    ///
    /// # Returns
    ///
    /// Returns the shell-specific install target.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Internal helper; use `install_shell_completion` to construct targets.
    /// ```
    fn new(shell: Shell, home_dir: &Path) -> ConfigResult<Self> {
        let target = match shell {
            Shell::Bash => Self {
                shell,
                completion_dir: home_dir.join(".bash_completion.d"),
                rc_path: Some(home_dir.join(".bashrc")),
            },
            Shell::Elvish => Self {
                shell,
                completion_dir: home_dir.join(".config").join("elvish").join("lib"),
                rc_path: Some(home_dir.join(".config").join("elvish").join("rc.elv")),
            },
            Shell::Fish => Self {
                shell,
                completion_dir: home_dir.join(".config").join("fish").join("completions"),
                rc_path: None,
            },
            Shell::PowerShell => Self {
                shell,
                completion_dir: home_dir
                    .join("Documents")
                    .join("PowerShell")
                    .join("Completions"),
                rc_path: Some(
                    home_dir
                        .join("Documents")
                        .join("PowerShell")
                        .join("Microsoft.PowerShell_profile.ps1"),
                ),
            },
            Shell::Zsh => Self {
                shell,
                completion_dir: home_dir.join(".zsh").join("completions"),
                rc_path: Some(home_dir.join(".zshrc")),
            },
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    format!("unsupported shell: {shell}"),
                )
                .into());
            }
        };

        Ok(target)
    }

    /// Builds the shell-specific startup block for a generated completion file.
    ///
    /// # Arguments
    ///
    /// - `generated_path`: Path to the generated completion file.
    /// - `completion_dir`: Directory containing generated completion files.
    ///
    /// # Returns
    ///
    /// Returns the startup-file block body, or `None` when the shell does not
    /// need startup-file changes.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Internal helper; use `install_shell_completion` to generate rc blocks.
    /// ```
    fn rc_block_body(&self, generated_path: &Path, completion_dir: &Path) -> Option<String> {
        let generated_path = generated_path.to_str()?;
        let completion_dir = completion_dir.to_str()?;

        let body = match self.shell {
            Shell::Bash => {
                format!("[[ -r \"{generated_path}\" ]] && source \"{generated_path}\"\n")
            }
            Shell::Elvish => format!("use {generated_path}\n"),
            Shell::PowerShell => {
                format!("if (Test-Path \"{generated_path}\") {{ . \"{generated_path}\" }}\n")
            }
            Shell::Zsh => format!(
                concat!(
                    "typeset -U fpath\n",
                    "fpath=(\"{}\" $fpath)\n",
                    "\n",
                    "autoload -Uz compinit\n",
                    "compinit\n",
                ),
                completion_dir,
            ),
            Shell::Fish => return None,
            _ => return None,
        };

        Some(body)
    }

    fn completion_file_path(&self, bin_name: &str) -> PathBuf {
        self.completion_dir.join(self.shell.file_name(bin_name))
    }

    fn managed_block_name(&self, bin_name: &str) -> String {
        match self.shell {
            Shell::Zsh => "rust-config-tree".to_owned(),
            _ => bin_name.to_owned(),
        }
    }
}

/// Inserts or replaces a managed shell configuration block in a startup file.
///
/// The managed block is identified by the binary name and shell, allowing repeat
/// installs to update the same block instead of appending duplicates.
/// Existing startup files are backed up before being modified.
///
/// # Arguments
///
/// - `bin_name`: Binary name used in the managed block markers.
/// - `shell`: Shell whose startup block is being inserted or replaced.
/// - `file_path`: Startup file to update.
/// - `block_body`: Shell-specific content placed between the managed markers.
///
/// # Returns
///
/// Returns `Ok(())` after the startup file has been written.
///
/// # Examples
///
/// ```
/// use std::fs;
/// use clap_complete::aot::Shell;
/// use rust_config_tree::upsert_managed_block;
///
/// let path = std::env::temp_dir().join("rust-config-tree-upsert-doctest.rc");
/// upsert_managed_block("myapp", Shell::Bash, &path, "body\n")?;
///
/// let content = fs::read_to_string(&path)?;
/// assert!(content.contains("# >>> myapp bash completions >>>"));
/// assert!(content.contains("body"));
/// # let _ = fs::remove_file(path);
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn upsert_managed_block(
    bin_name: &str,
    shell: Shell,
    file_path: &Path,
    block_body: &str,
) -> io::Result<()> {
    upsert_managed_block_with_backup_name(bin_name, shell, file_path, block_body, bin_name)
}

fn upsert_managed_block_with_backup_name(
    block_name: &str,
    shell: Shell,
    file_path: &Path,
    block_body: &str,
    backup_name: &str,
) -> io::Result<()> {
    let (begin_marker, end_marker) = managed_block_markers(block_name, shell);

    let existing = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(err) if err.kind() == io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(err),
    };

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let managed_block = format!("{begin_marker}\n{block_body}\n{end_marker}\n");

    let next_content = if let Some(begin_pos) = existing.find(&begin_marker) {
        if let Some(relative_end_pos) = existing[begin_pos..].find(&end_marker) {
            let end_pos = begin_pos + relative_end_pos + end_marker.len();

            let before = existing[..begin_pos].trim_end();
            let after = existing[end_pos..].trim_start();

            match (before.is_empty(), after.is_empty()) {
                (true, true) => managed_block,
                (true, false) => format!("{managed_block}\n{after}"),
                (false, true) => format!("{before}\n\n{managed_block}"),
                (false, false) => format!("{before}\n\n{managed_block}\n{after}"),
            }
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("found `{begin_marker}` but missing `{end_marker}`"),
            ));
        }
    } else {
        let existing = existing.trim_end();

        if existing.is_empty() {
            managed_block
        } else {
            format!("{existing}\n\n{managed_block}")
        }
    };

    write_startup_file_if_changed(file_path, &existing, next_content, backup_name)
}

#[cfg(test)]
fn remove_managed_block(bin_name: &str, shell: Shell, file_path: &Path) -> io::Result<bool> {
    remove_managed_block_with_backup_name(bin_name, shell, file_path, bin_name)
}

fn remove_managed_block_with_backup_name(
    block_name: &str,
    shell: Shell,
    file_path: &Path,
    backup_name: &str,
) -> io::Result<bool> {
    let (begin_marker, end_marker) = managed_block_markers(block_name, shell);

    let existing = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(err) => return Err(err),
    };

    let Some(begin_pos) = existing.find(&begin_marker) else {
        return Ok(false);
    };

    let Some(relative_end_pos) = existing[begin_pos..].find(&end_marker) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("found `{begin_marker}` but missing `{end_marker}`"),
        ));
    };

    let end_pos = begin_pos + relative_end_pos + end_marker.len();
    let before = existing[..begin_pos].trim_end();
    let after = existing[end_pos..].trim_start();

    let next_content = match (before.is_empty(), after.is_empty()) {
        (true, true) => String::new(),
        (true, false) => after.to_owned(),
        (false, true) => format!("{before}\n"),
        (false, false) => format!("{before}\n\n{after}"),
    };

    write_startup_file_if_changed(file_path, &existing, next_content, backup_name)?;
    Ok(true)
}

fn remove_completion_file(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

fn completion_dir_is_empty(path: &Path) -> io::Result<bool> {
    match fs::read_dir(path) {
        Ok(mut entries) => Ok(entries.next().is_none()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(true),
        Err(err) => Err(err),
    }
}

fn managed_block_markers(block_name: &str, shell: Shell) -> (String, String) {
    (
        format!("# >>> {block_name} {shell} completions >>>"),
        format!("# <<< {block_name} {shell} completions <<<"),
    )
}

fn write_startup_file_if_changed(
    file_path: &Path,
    existing: &str,
    next_content: String,
    backup_name: &str,
) -> io::Result<()> {
    if existing == next_content {
        return Ok(());
    }

    if !existing.is_empty() || file_path.exists() {
        backup_startup_file(file_path, backup_name)?;
    }

    fs::write(file_path, next_content)
}

fn backup_startup_file(file_path: &Path, backup_name: &str) -> io::Result<PathBuf> {
    let file_name = file_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "startup file path does not have a valid UTF-8 file name",
            )
        })?;
    let backup_name = backup_file_name_part(backup_name);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?
        .as_nanos();
    let backup_file_name = format!("{file_name}.backup.by.{backup_name}.{timestamp}");
    let backup_path = file_path.with_file_name(backup_file_name);

    fs::copy(file_path, &backup_path)?;
    Ok(backup_path)
}

fn backup_file_name_part(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' => ch,
            _ => '_',
        })
        .collect()
}

#[cfg(test)]
#[path = "unit_tests/cli.rs"]
mod unit_tests;
