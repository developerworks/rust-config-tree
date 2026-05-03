//! High-level runtime loading through Figment and `confique`.
//!
//! This module is responsible for discovering the recursive include tree,
//! merging config files in runtime precedence order, layering schema-declared
//! environment variables on top, and finally asking `confique` to apply
//! defaults and validation.

use std::path::Path;

use confique::{Config, Layer};
use figment::{
    Figment,
    providers::{Format, Json, Toml, Yaml},
};

use crate::{
    ConfigSource, ConfigTree, ConfigTreeOptions, IncludeOrder, absolutize_lexical,
    config::{ConfigResult, ConfigSchema},
    config_env::ConfiqueEnvProvider,
    config_format::ConfigFormat,
    config_trace::trace_config_sources,
};

/// Loads a complete `confique` schema from a root config path.
///
/// The loader follows recursive include paths exposed by [`ConfigSchema`],
/// resolves relative include paths from the declaring file, detects include
/// cycles, loads the first `.env` file found from the root config directory
/// upward, builds a [`Figment`] from config files and schema-declared
/// environment variables, and then asks `confique` to apply defaults and
/// validation. Existing process environment variables take precedence over
/// values loaded from `.env`.
///
/// # Type Parameters
///
/// - `S`: Config schema type that derives [`Config`] and implements
///   [`ConfigSchema`].
///
/// # Arguments
///
/// - `path`: Root config file path.
///
/// # Returns
///
/// Returns the merged config schema after loading the root file, recursive
/// includes, `.env` values, and environment values.
///
/// # Examples
///
/// ```
/// use std::fs;
/// use confique::Config;
/// use rust_config_tree::{ConfigSchema, load_config};
///
/// #[derive(Debug, Config)]
/// struct AppConfig {
///     #[config(default = [])]
///     include: Vec<std::path::PathBuf>,
///     #[config(default = "demo")]
///     mode: String,
/// }
///
/// impl ConfigSchema for AppConfig {
///     fn include_paths(layer: &<Self as Config>::Layer) -> Vec<std::path::PathBuf> {
///         layer.include.clone().unwrap_or_default()
///     }
/// }
///
/// # fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
/// let path = std::env::temp_dir().join("rust-config-tree-load-config-doctest.yaml");
/// fs::write(&path, "mode: local\n")?;
///
/// let config = load_config::<AppConfig>(&path)?;
///
/// assert_eq!(config.mode, "local");
/// # let _ = fs::remove_file(path);
/// # Ok(())
/// # }
/// # run().unwrap();
/// ```
pub fn load_config<S>(path: impl AsRef<Path>) -> ConfigResult<S>
where
    S: ConfigSchema,
{
    let (config, _) = load_config_with_figment::<S>(path)?;
    Ok(config)
}

/// Loads a config schema and returns the Figment graph used for runtime loading.
///
/// The returned [`Figment`] can be inspected with [`Figment::find_metadata`] to
/// determine which provider supplied a runtime value.
///
/// # Type Parameters
///
/// - `S`: Config schema type that derives [`Config`] and implements
///   [`ConfigSchema`].
///
/// # Arguments
///
/// - `path`: Root config file path.
///
/// # Returns
///
/// Returns the merged config schema and its runtime Figment source graph.
///
/// # Examples
///
/// ```
/// use std::fs;
/// use confique::Config;
/// use rust_config_tree::{ConfigSchema, load_config_with_figment};
///
/// #[derive(Debug, Config)]
/// struct AppConfig {
///     #[config(default = [])]
///     include: Vec<std::path::PathBuf>,
///     #[config(default = "demo")]
///     mode: String,
/// }
///
/// impl ConfigSchema for AppConfig {
///     fn include_paths(layer: &<Self as Config>::Layer) -> Vec<std::path::PathBuf> {
///         layer.include.clone().unwrap_or_default()
///     }
/// }
///
/// # fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
/// let path = std::env::temp_dir().join("rust-config-tree-load-with-figment-doctest.yaml");
/// fs::write(&path, "mode: local\n")?;
///
/// let (config, figment) = load_config_with_figment::<AppConfig>(&path)?;
///
/// assert_eq!(config.mode, "local");
/// # let _ = figment;
/// # let _ = fs::remove_file(path);
/// # Ok(())
/// # }
/// # run().unwrap();
/// ```
pub fn load_config_with_figment<S>(path: impl AsRef<Path>) -> ConfigResult<(S, Figment)>
where
    S: ConfigSchema,
{
    let figment = build_config_figment::<S>(path)?;
    let config = load_config_from_figment::<S>(&figment)?;

    Ok((config, figment))
}

/// Builds the Figment runtime source graph for a config tree.
///
/// Config files are merged in include order, then environment variables
/// declared by [`ConfiqueEnvProvider`] are merged with higher priority.
///
/// # Type Parameters
///
/// - `S`: Config schema type used to discover includes and environment names.
///
/// # Arguments
///
/// - `path`: Root config file path.
///
/// # Returns
///
/// Returns a Figment source graph with file and environment providers.
///
/// # Examples
///
/// ```
/// use std::fs;
/// use confique::Config;
/// use rust_config_tree::{ConfigSchema, build_config_figment};
///
/// #[derive(Debug, Config)]
/// struct AppConfig {
///     #[config(default = [])]
///     include: Vec<std::path::PathBuf>,
///     #[config(default = "demo")]
///     mode: String,
/// }
///
/// impl ConfigSchema for AppConfig {
///     fn include_paths(layer: &<Self as Config>::Layer) -> Vec<std::path::PathBuf> {
///         layer.include.clone().unwrap_or_default()
///     }
/// }
///
/// # fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
/// let path = std::env::temp_dir().join("rust-config-tree-build-figment-doctest.yaml");
/// fs::write(&path, "mode: local\n")?;
///
/// let figment = build_config_figment::<AppConfig>(&path)?;
/// # let _ = figment;
/// # let _ = fs::remove_file(path);
/// # Ok(())
/// # }
/// # run().unwrap();
/// ```
pub fn build_config_figment<S>(path: impl AsRef<Path>) -> ConfigResult<Figment>
where
    S: ConfigSchema,
{
    let path = path.as_ref();
    load_dotenv_for_path(path)?;

    let tree = load_layer_tree::<S>(path)?;
    let mut figment = Figment::new();

    for node in tree.nodes().iter().rev() {
        figment = merge_file_provider(figment, node.path());
    }

    Ok(figment.merge(ConfiqueEnvProvider::new::<S>()))
}

/// Extracts and validates a config schema from a Figment source graph.
///
/// Figment supplies runtime values. `confique` supplies code defaults and final
/// validation.
///
/// # Type Parameters
///
/// - `S`: Config schema type to extract and validate.
///
/// # Arguments
///
/// - `figment`: Runtime source graph.
///
/// # Returns
///
/// Returns the final config schema.
///
/// # Examples
///
/// ```
/// use std::fs;
/// use confique::Config;
/// use rust_config_tree::{ConfigSchema, build_config_figment, load_config_from_figment};
///
/// #[derive(Debug, Config)]
/// struct AppConfig {
///     #[config(default = [])]
///     include: Vec<std::path::PathBuf>,
///     #[config(default = "demo")]
///     mode: String,
/// }
///
/// impl ConfigSchema for AppConfig {
///     fn include_paths(layer: &<Self as Config>::Layer) -> Vec<std::path::PathBuf> {
///         layer.include.clone().unwrap_or_default()
///     }
/// }
///
/// # fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
/// let path = std::env::temp_dir().join("rust-config-tree-load-from-figment-doctest.yaml");
/// fs::write(&path, "mode: local\n")?;
/// let figment = build_config_figment::<AppConfig>(&path)?;
///
/// let config = load_config_from_figment::<AppConfig>(&figment)?;
///
/// assert_eq!(config.mode, "local");
/// # let _ = fs::remove_file(path);
/// # Ok(())
/// # }
/// # run().unwrap();
/// ```
pub fn load_config_from_figment<S>(figment: &Figment) -> ConfigResult<S>
where
    S: ConfigSchema,
{
    let runtime_layer: <S as Config>::Layer = figment.extract()?;
    let config = S::from_layer(runtime_layer.with_fallback(S::Layer::default_values()))?;

    trace_config_sources::<S>(figment);

    Ok(config)
}

/// Loads one config layer from disk using the format inferred from the path.
///
/// # Type Parameters
///
/// - `S`: Config schema type whose intermediate `confique` layer should be
///   loaded.
///
/// # Arguments
///
/// - `path`: Config file path to load.
///
/// # Returns
///
/// Returns the loaded `confique` layer for `S`.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
pub(crate) fn load_layer<S>(path: &Path) -> ConfigResult<<S as Config>::Layer>
where
    S: ConfigSchema,
{
    Ok(figment_for_file(path).extract()?)
}

/// Loads every config layer reachable from the root include tree.
///
/// # Type Parameters
///
/// - `S`: Config schema type whose layer type is loaded for each file.
///
/// # Arguments
///
/// - `path`: Root config path used to start include traversal.
///
/// # Returns
///
/// Returns the loaded config tree containing one `confique` layer per source.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn load_layer_tree<S>(path: &Path) -> ConfigResult<ConfigTree<<S as Config>::Layer>>
where
    S: ConfigSchema,
{
    // Reverse traversal lets later declared includes override earlier files
    // after the collected nodes are merged from leaves back toward the root.
    Ok(ConfigTreeOptions::default()
        .include_order(IncludeOrder::Reverse)
        .load(
            path,
            |path| -> ConfigResult<ConfigSource<<S as Config>::Layer>> {
                let layer = load_layer::<S>(path)?;
                let include_paths = S::include_paths(&layer);
                Ok(ConfigSource::new(layer, include_paths))
            },
        )?)
}

/// Merges one file provider selected from the path extension.
///
/// # Arguments
///
/// - `figment`: Existing Figment graph to extend.
/// - `path`: Config file path whose extension selects the provider format.
///
/// # Returns
///
/// Returns `figment` with the selected file provider merged in.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn merge_file_provider(figment: Figment, path: &Path) -> Figment {
    match ConfigFormat::from_path(path) {
        ConfigFormat::Yaml => figment.merge(Yaml::file_exact(path)),
        ConfigFormat::Toml => figment.merge(Toml::file_exact(path)),
        ConfigFormat::Json => figment.merge(Json::file_exact(path)),
    }
}

/// Builds a Figment graph containing only one config file provider.
///
/// # Arguments
///
/// - `path`: Config file path to load through Figment.
///
/// # Returns
///
/// Returns a Figment graph containing exactly that file provider.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
pub(crate) fn figment_for_file(path: &Path) -> Figment {
    merge_file_provider(Figment::new(), path)
}

/// Loads the nearest ancestor `.env` file for a config path when it exists.
///
/// # Arguments
///
/// - `path`: Config file path whose ancestors should be searched.
///
/// # Returns
///
/// Returns `Ok(())` after loading the first discovered `.env`, or when none
/// exists.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn load_dotenv_for_path(path: &Path) -> ConfigResult<()> {
    let path = absolutize_lexical(path)?;
    let mut current_dir = path.parent();

    while let Some(dir) = current_dir {
        let dotenv_path = dir.join(".env");
        if dotenv_path.try_exists()? {
            // `dotenvy` preserves existing process variables, so explicit
            // environment values keep precedence over values from `.env`.
            dotenvy::from_path(&dotenv_path)?;
            break;
        }
        current_dir = dir.parent();
    }

    Ok(())
}
