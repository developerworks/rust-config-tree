//! Clap subcommand integration and shell completion installation helpers.
//!
//! This module exposes reusable commands for generating config templates,
//! printing shell completions, and installing completions into common shell
//! startup locations.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use clap::{CommandFactory, Subcommand};
use clap_complete::aot::{Shell, generate, generate_to};

use crate::{
    ConfigResult, ConfigSchema,
    config::{resolve_config_template_output, write_config_templates},
};

/// Built-in clap subcommands for config templates and shell completions.
#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// Generate an example config template.
    ///
    /// The output format is inferred from the extension; unknown or missing extensions use YAML.
    ConfigTemplate {
        /// Template output path. Defaults to `config.example.yaml`.
        #[arg(long)]
        output: Option<PathBuf>,
    },

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
}

/// Handles a built-in config subcommand for a consumer CLI.
///
/// `C` is the clap parser type used to generate completion metadata. `S` is the
/// application config schema used for template generation.
///
/// # Type Parameters
///
/// - `C`: The consumer CLI parser type that implements [`CommandFactory`].
/// - `S`: The consumer config schema used when rendering config templates.
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
pub fn handle_config_command<C, S>(command: ConfigCommand, config_path: &Path) -> ConfigResult<()>
where
    C: CommandFactory,
    S: ConfigSchema,
{
    match command {
        ConfigCommand::ConfigTemplate { output } => {
            let output = resolve_config_template_output(output)?;
            write_config_templates::<S>(config_path, output)
        }
        ConfigCommand::Completions { shell } => {
            print_shell_completion::<C>(shell);
            Ok(())
        }
        ConfigCommand::InstallCompletions { shell } => install_shell_completion::<C>(shell),
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
        upsert_managed_block(&bin_name, shell, rc_path, &block_body)?;
        println!("{shell} rc configured: {}", rc_path.display());
    }

    println!("{shell} completion generated: {}", generated_path.display());
    println!("restart {shell} or open a new shell session");

    Ok(())
}

/// Resolves the current user's home directory from environment variables.
///
/// # Returns
///
/// Returns the home directory when `HOME` or `USERPROFILE` is set.
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
}

/// Inserts or replaces a managed shell configuration block in a startup file.
///
/// The managed block is identified by the binary name and shell, allowing repeat
/// installs to update the same block instead of appending duplicates.
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
pub fn upsert_managed_block(
    bin_name: &str,
    shell: Shell,
    file_path: &Path,
    block_body: &str,
) -> io::Result<()> {
    let begin_marker = format!("# >>> {bin_name} {shell} completions >>>");
    let end_marker = format!("# <<< {bin_name} {shell} completions <<<");

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

    fs::write(file_path, next_content)
}

#[cfg(test)]
#[path = "unit_tests/cli.rs"]
mod unit_tests;
