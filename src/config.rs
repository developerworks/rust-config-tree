//! High-level `confique` integration and config-template rendering.
//!
//! This module loads `.env` values, builds a Figment runtime source graph,
//! extracts it into a `confique` schema for defaults and validation, renders
//! example templates that mirror the same include tree, and writes JSON Schema
//! files that editors can use for completion and validation. YAML templates can
//! also be split across nested schema sections.

use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    path::Component,
    path::{Path, PathBuf},
    sync::Arc,
};

use confique::{
    Config, FileFormat, Layer,
    meta::{Expr, FieldKind, LeafKind, MapKey, Meta},
};
use figment::{
    Figment, Metadata, Profile, Provider, Source,
    providers::{Env, Format, Json, Toml, Yaml},
    value::{Dict, Map, Uncased},
};
use schemars::{JsonSchema, generate::SchemaSettings};
use tracing::trace;

use crate::{
    ConfigError, ConfigSource, ConfigTree, ConfigTreeOptions, IncludeOrder, absolutize_lexical,
    collect_template_targets, normalize_lexical, select_template_source,
};

/// Result type used by the high-level configuration API.
///
/// The error type is [`ConfigError`].
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;

/// A `confique` schema that can expose recursive include paths and template
/// section layout.
///
/// Implement this trait for the same type that derives `confique::Config`.
/// `include_paths` receives a partially loaded layer so the crate can discover
/// child config files before the final schema is merged.
pub trait ConfigSchema: Config + Sized {
    /// Returns include paths declared by a loaded config layer.
    ///
    /// Relative paths are resolved from the file that declared them. Empty paths
    /// are rejected before traversal continues.
    ///
    /// # Arguments
    ///
    /// - `layer`: Partially loaded `confique` layer for one config file.
    ///
    /// # Returns
    ///
    /// Returns include paths declared by `layer`.
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf>;

    /// Overrides the generated template file path for a nested section.
    ///
    /// By default, top-level sections are generated as `config/<field>.yaml`
    /// and nested sections as children of their parent section file stem, e.g.
    /// `config/trading/risk.yaml`.
    ///
    /// # Arguments
    ///
    /// - `section_path`: Path of nested schema field names from the root schema
    ///   to the section being rendered.
    ///
    /// # Returns
    ///
    /// Returns `Some(path)` to override the generated file path, or `None` to
    /// use the default section path.
    fn template_path_for_section(section_path: &[&str]) -> Option<PathBuf> {
        let _ = section_path;
        None
    }
}

/// File format used when loading config files or rendering templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    /// YAML format, selected for `.yaml`, `.yml`, unknown extensions, and paths
    /// without an extension.
    Yaml,
    /// TOML format, selected for `.toml`.
    Toml,
    /// JSON5-compatible format, selected for `.json` and `.json5`.
    Json,
}

impl ConfigFormat {
    /// Infers the config format from a path extension.
    ///
    /// Unknown extensions intentionally fall back to YAML.
    ///
    /// # Arguments
    ///
    /// - `path`: Config or template path whose extension should be inspected.
    ///
    /// # Returns
    ///
    /// Returns the inferred [`ConfigFormat`].
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        match path.as_ref().extension().and_then(OsStr::to_str) {
            Some("toml") => Self::Toml,
            Some("json" | "json5") => Self::Json,
            Some("yaml" | "yml") | Some(_) | None => Self::Yaml,
        }
    }

    /// Converts this format into the `confique` file format used for loading.
    ///
    /// # Returns
    ///
    /// Returns the matching [`FileFormat`] value.
    pub fn as_file_format(self) -> FileFormat {
        match self {
            Self::Yaml => FileFormat::Yaml,
            Self::Toml => FileFormat::Toml,
            Self::Json => FileFormat::Json5,
        }
    }
}

/// Generated template content for one output path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigTemplateTarget {
    /// Path that should receive the generated content.
    pub path: PathBuf,
    /// Complete template content to write to `path`.
    pub content: String,
}

/// Figment provider that maps environment variables declared in `confique`
/// schema metadata onto their exact field paths.
///
/// This provider reads `#[config(env = "...")]` from [`Config::META`] and
/// avoids Figment's delimiter-based environment key splitting. Environment
/// variables such as `APP_DATABASE_POOL_SIZE` can therefore map to a Rust field
/// named `database.pool_size` without treating the single underscores as nested
/// separators.
#[derive(Clone)]
pub struct ConfiqueEnvProvider {
    env: Env,
    path_to_env: Arc<HashMap<String, String>>,
}

impl ConfiqueEnvProvider {
    /// Creates an environment provider for a `confique` schema.
    ///
    /// # Type Parameters
    ///
    /// - `S`: Config schema whose metadata declares environment variable names.
    ///
    /// # Returns
    ///
    /// Returns a provider that emits only environment variables declared by `S`.
    pub fn new<S>() -> Self
    where
        S: Config,
    {
        let mut env_to_path = HashMap::<String, String>::new();
        let mut path_to_env = HashMap::<String, String>::new();

        collect_env_mapping(&S::META, "", &mut env_to_path, &mut path_to_env);

        let env_to_path = Arc::new(env_to_path);
        let path_to_env = Arc::new(path_to_env);
        let map_for_filter = Arc::clone(&env_to_path);

        let env = Env::raw().filter_map(move |env_key| {
            let lookup_key = env_key.as_str().to_ascii_uppercase();

            map_for_filter
                .get(&lookup_key)
                .cloned()
                .map(Uncased::from_owned)
        });

        Self { env, path_to_env }
    }
}

impl Provider for ConfiqueEnvProvider {
    fn metadata(&self) -> Metadata {
        let path_to_env = Arc::clone(&self.path_to_env);

        Metadata::named("environment variable").interpolater(move |_profile, keys| {
            let path = keys.join(".");

            path_to_env.get(&path).cloned().unwrap_or(path)
        })
    }

    fn data(&self) -> Result<Map<Profile, Dict>, figment::Error> {
        self.env.data()
    }
}

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
pub fn load_layer<S>(path: &Path) -> ConfigResult<<S as Config>::Layer>
where
    S: ConfigSchema,
{
    Ok(figment_for_file(path).extract()?)
}

fn load_layer_tree<S>(path: &Path) -> ConfigResult<ConfigTree<<S as Config>::Layer>>
where
    S: ConfigSchema,
{
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

fn merge_file_provider(figment: Figment, path: &Path) -> Figment {
    match ConfigFormat::from_path(path) {
        ConfigFormat::Yaml => figment.merge(Yaml::file_exact(path)),
        ConfigFormat::Toml => figment.merge(Toml::file_exact(path)),
        ConfigFormat::Json => figment.merge(Json::file_exact(path)),
    }
}

fn figment_for_file(path: &Path) -> Figment {
    merge_file_provider(Figment::new(), path)
}

/// Writes a Draft 7 JSON Schema for a config type.
///
/// The same generated schema can be referenced from TOML, YAML, and JSON
/// configuration files. TOML and YAML templates can bind it with editor
/// directives. JSON files should usually be bound through editor settings
/// rather than a runtime `$schema` field.
///
/// # Type Parameters
///
/// - `S`: Config schema type that derives [`JsonSchema`].
///
/// # Arguments
///
/// - `output_path`: Destination path for the generated JSON Schema.
///
/// # Returns
///
/// Returns `Ok(())` after the schema file has been written.
pub fn write_config_schema<S>(output_path: impl AsRef<Path>) -> ConfigResult<()>
where
    S: JsonSchema,
{
    let generator = SchemaSettings::draft07().into_generator();
    let schema = generator.into_root_schema_for::<S>();
    let mut json = serde_json::to_string_pretty(&schema)?;
    ensure_single_trailing_newline(&mut json);

    write_template(output_path.as_ref(), &json)
}

/// Renders the default template for one path.
///
/// The template format is inferred from the path extension.
///
/// # Type Parameters
///
/// - `S`: Config schema type used to render the template.
///
/// # Arguments
///
/// - `path`: Output path whose extension selects the template format.
///
/// # Returns
///
/// Returns the generated template content.
pub fn template_for_path<S>(path: impl AsRef<Path>) -> ConfigResult<String>
where
    S: ConfigSchema,
{
    let template = match ConfigFormat::from_path(path.as_ref()) {
        ConfigFormat::Yaml => confique::yaml::template::<S>(yaml_options()),
        ConfigFormat::Toml => confique::toml::template::<S>(toml_options()),
        ConfigFormat::Json => confique::json5::template::<S>(json5_options()),
    };

    Ok(template)
}

/// Collects all template targets that should be generated for a config tree.
///
/// The root template source is selected with [`select_template_source`]. Include
/// paths found in the source tree are mirrored under `output_path` for relative
/// includes. When a source node has no includes, nested `confique` sections are
/// used to derive child template files with paths from
/// [`ConfigSchema::template_path_for_section`].
///
/// # Type Parameters
///
/// - `S`: Config schema type used to discover includes and render templates.
///
/// # Arguments
///
/// - `config_path`: Root config path preferred as the template source when it
///   exists.
/// - `output_path`: Root output path for generated templates.
///
/// # Returns
///
/// Returns all generated template targets in traversal order.
pub fn template_targets_for_paths<S>(
    config_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> ConfigResult<Vec<ConfigTemplateTarget>>
where
    S: ConfigSchema,
{
    let output_path = output_path.as_ref();
    let source_path = select_template_source(config_path, output_path);
    let root_source_path = absolutize_lexical(source_path)?;
    let output_base_dir = output_path.parent().unwrap_or_else(|| Path::new("."));

    let template_targets = collect_template_targets(
        &root_source_path,
        output_path,
        |node_source_path| -> ConfigResult<Vec<PathBuf>> {
            let mut include_paths = template_source_include_paths::<S>(node_source_path)?;

            if include_paths.is_empty() {
                include_paths =
                    default_child_include_paths::<S>(&root_source_path, node_source_path);
            }

            Ok(include_paths)
        },
    )?;

    let split_paths = template_targets
        .iter()
        .filter_map(|target| {
            section_path_for_target::<S>(output_base_dir, target.target_path())
                .filter(|section_path| !section_path.is_empty())
        })
        .collect::<Vec<_>>();

    template_targets
        .into_iter()
        .map(|target| {
            let (_, target_path, include_paths) = target.into_parts();
            let section_path =
                section_path_for_target::<S>(output_base_dir, &target_path).unwrap_or_default();
            Ok(ConfigTemplateTarget {
                content: template_for_target::<S>(
                    &target_path,
                    &include_paths,
                    &section_path,
                    &split_paths,
                )?,
                path: target_path,
            })
        })
        .collect()
}

/// Collects template targets and binds TOML/YAML templates to a JSON Schema.
///
/// TOML targets receive a `#:schema` directive. YAML targets receive a YAML
/// Language Server modeline. JSON and JSON5 targets are left unchanged so the
/// runtime configuration is not polluted with a `$schema` field.
///
/// # Type Parameters
///
/// - `S`: Config schema type used to discover includes and render templates.
///
/// # Arguments
///
/// - `config_path`: Root config path preferred as the template source when it
///   exists.
/// - `output_path`: Root output path for generated templates.
/// - `schema_path`: JSON Schema path to reference from TOML/YAML templates.
///
/// # Returns
///
/// Returns all generated template targets in traversal order.
pub fn template_targets_for_paths_with_schema<S>(
    config_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    schema_path: impl AsRef<Path>,
) -> ConfigResult<Vec<ConfigTemplateTarget>>
where
    S: ConfigSchema,
{
    template_targets_for_paths::<S>(config_path, output_path)?
        .into_iter()
        .map(|mut target| {
            target.content = template_with_schema_directive(
                &target.path,
                schema_path.as_ref(),
                &target.content,
            )?;
            Ok(target)
        })
        .collect()
}

/// Writes all generated config templates for a config tree.
///
/// Parent directories are created before each target is written.
///
/// # Type Parameters
///
/// - `S`: Config schema type used to discover includes and render templates.
///
/// # Arguments
///
/// - `config_path`: Root config path preferred as the template source when it
///   exists.
/// - `output_path`: Root output path for generated templates.
///
/// # Returns
///
/// Returns `Ok(())` after all template files have been written.
pub fn write_config_templates<S>(
    config_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> ConfigResult<()>
where
    S: ConfigSchema,
{
    for target in template_targets_for_paths::<S>(config_path, output_path)? {
        write_template(&target.path, &target.content)?;
    }

    Ok(())
}

/// Writes all generated config templates with editor schema bindings.
///
/// TOML targets receive `#:schema <path>`, YAML targets receive
/// `# yaml-language-server: $schema=<path>`, and JSON targets are left
/// unchanged. The schema path is rendered relative to each template file.
///
/// # Type Parameters
///
/// - `S`: Config schema type used to discover includes and render templates.
///
/// # Arguments
///
/// - `config_path`: Root config path preferred as the template source when it
///   exists.
/// - `output_path`: Root output path for generated templates.
/// - `schema_path`: JSON Schema path to reference from TOML/YAML templates.
///
/// # Returns
///
/// Returns `Ok(())` after all template files have been written.
pub fn write_config_templates_with_schema<S>(
    config_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    schema_path: impl AsRef<Path>,
) -> ConfigResult<()>
where
    S: ConfigSchema,
{
    for target in
        template_targets_for_paths_with_schema::<S>(config_path, output_path, schema_path)?
    {
        write_template(&target.path, &target.content)?;
    }

    Ok(())
}

/// Writes one generated template file, creating parent directories first.
///
/// # Arguments
///
/// - `path`: Destination file path.
/// - `content`: Complete template content to write.
///
/// # Returns
///
/// Returns `Ok(())` after the file has been written.
pub(crate) fn write_template(path: &Path, content: &str) -> ConfigResult<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content)?;
    Ok(())
}

/// Resolves the CLI template output path to a normalized absolute path.
///
/// # Arguments
///
/// - `output`: Optional user-provided output path. When omitted,
///   `config.example.yaml` is used.
///
/// # Returns
///
/// Returns a normalized absolute output path.
pub(crate) fn resolve_config_template_output(output: Option<PathBuf>) -> ConfigResult<PathBuf> {
    let current_dir = std::env::current_dir()?;
    let output = output.unwrap_or_else(|| PathBuf::from("config.example.yaml"));
    let output = if output.is_absolute() {
        output
    } else {
        current_dir.join(output)
    };

    Ok(normalize_lexical(output))
}

fn template_source_include_paths<S>(path: &Path) -> ConfigResult<Vec<PathBuf>>
where
    S: ConfigSchema,
{
    if !path.exists() {
        return Ok(Vec::new());
    }

    match load_layer::<S>(path) {
        Ok(layer) => Ok(S::include_paths(&layer)),
        Err(_) => load_include_paths_only(path),
    }
}

fn load_include_paths_only(path: &Path) -> ConfigResult<Vec<PathBuf>> {
    match figment_for_file(path).extract_inner::<Vec<PathBuf>>("include") {
        Ok(paths) => Ok(paths),
        Err(error) if error.missing() => Ok(Vec::new()),
        Err(error) => Err(error.into()),
    }
}

fn template_with_schema_directive(
    template_path: &Path,
    schema_path: &Path,
    content: &str,
) -> ConfigResult<String> {
    let schema_ref = schema_reference_for_path(template_path, schema_path)?;
    let directive = match ConfigFormat::from_path(template_path) {
        ConfigFormat::Yaml => Some(format!("# yaml-language-server: $schema={schema_ref}")),
        ConfigFormat::Toml => Some(format!("#:schema {schema_ref}")),
        ConfigFormat::Json => None,
    };

    let Some(directive) = directive else {
        return Ok(content.to_owned());
    };

    Ok(format!("{directive}\n\n{content}"))
}

fn schema_reference_for_path(template_path: &Path, schema_path: &Path) -> ConfigResult<String> {
    let template_path = absolutize_lexical(template_path)?;
    let schema_path = absolutize_lexical(schema_path)?;
    let template_dir = template_path.parent().unwrap_or_else(|| Path::new("."));
    let relative_path = relative_path_from(&schema_path, template_dir);
    Ok(render_schema_reference(&relative_path))
}

fn relative_path_from(path: &Path, base: &Path) -> PathBuf {
    let path_components = path.components().collect::<Vec<_>>();
    let base_components = base.components().collect::<Vec<_>>();

    let mut common_len = 0;
    while common_len < path_components.len()
        && common_len < base_components.len()
        && path_components[common_len] == base_components[common_len]
    {
        common_len += 1;
    }

    if common_len == 0 {
        return path.to_path_buf();
    }

    let mut relative = PathBuf::new();
    for component in &base_components[common_len..] {
        if matches!(component, Component::Normal(_)) {
            relative.push("..");
        }
    }

    for component in &path_components[common_len..] {
        relative.push(component.as_os_str());
    }

    if relative.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        relative
    }
}

fn render_schema_reference(path: &Path) -> String {
    let value = path.to_string_lossy().replace('\\', "/");
    if path.is_absolute() || value.starts_with("../") || value.starts_with("./") {
        value
    } else {
        format!("./{value}")
    }
}

fn template_for_target<S>(
    path: &Path,
    include_paths: &[PathBuf],
    section_path: &[&'static str],
    split_paths: &[Vec<&'static str>],
) -> ConfigResult<String>
where
    S: ConfigSchema,
{
    if ConfigFormat::from_path(path) != ConfigFormat::Yaml || split_paths.is_empty() {
        return template_for_path_with_includes::<S>(path, include_paths);
    }

    Ok(render_yaml_template(
        &S::META,
        include_paths,
        section_path,
        split_paths,
    ))
}

fn default_child_include_paths<S>(root_source_path: &Path, node_source_path: &Path) -> Vec<PathBuf>
where
    S: ConfigSchema,
{
    let root_base_dir = root_source_path.parent().unwrap_or_else(|| Path::new("."));
    let section_path =
        section_path_for_target::<S>(root_base_dir, node_source_path).unwrap_or_default();
    let source_base_dir = node_source_path.parent().unwrap_or_else(|| Path::new("."));

    immediate_child_section_paths(&S::META, &section_path)
        .into_iter()
        .map(|child_section_path| {
            let child_path =
                root_base_dir.join(template_path_for_section::<S>(&child_section_path));
            path_relative_to(&child_path, source_base_dir)
        })
        .collect()
}

fn collect_env_mapping(
    meta: &'static Meta,
    prefix: &str,
    env_to_path: &mut HashMap<String, String>,
    path_to_env: &mut HashMap<String, String>,
) {
    for field in meta.fields {
        let path = if prefix.is_empty() {
            field.name.to_owned()
        } else {
            format!("{prefix}.{}", field.name)
        };

        match field.kind {
            FieldKind::Leaf { env: Some(env), .. } => {
                env_to_path.insert(env.to_ascii_uppercase(), path.clone());
                path_to_env.insert(path, env.to_owned());
            }
            FieldKind::Leaf { env: None, .. } => {}
            FieldKind::Nested { meta } => {
                collect_env_mapping(meta, &path, env_to_path, path_to_env);
            }
        }
    }
}

fn load_dotenv_for_path(path: &Path) -> ConfigResult<()> {
    let path = absolutize_lexical(path)?;
    let mut current_dir = path.parent();

    while let Some(dir) = current_dir {
        let dotenv_path = dir.join(".env");
        if dotenv_path.try_exists()? {
            dotenvy::from_path(&dotenv_path)?;
            break;
        }
        current_dir = dir.parent();
    }

    Ok(())
}

fn section_path_for_target<S>(root_base_dir: &Path, target_path: &Path) -> Option<Vec<&'static str>>
where
    S: ConfigSchema,
{
    let normalized_target = normalize_lexical(target_path);

    for section_path in nested_section_paths(&S::META) {
        let section_target =
            normalize_lexical(root_base_dir.join(template_path_for_section::<S>(&section_path)));
        if section_target == normalized_target {
            return Some(section_path);
        }
    }

    infer_section_path_from_path::<S>(target_path)
}

fn template_path_for_section<S>(section_path: &[&str]) -> PathBuf
where
    S: ConfigSchema,
{
    if let Some(path) = S::template_path_for_section(section_path) {
        return path;
    }

    let Some((last, parent_path)) = section_path.split_last() else {
        return PathBuf::new();
    };

    if parent_path.is_empty() {
        return PathBuf::from("config").join(format!("{last}.yaml"));
    }

    let parent_template_path = template_path_for_section::<S>(parent_path);
    parent_template_path
        .with_extension("")
        .join(format!("{last}.yaml"))
}

fn path_relative_to(path: &Path, base: &Path) -> PathBuf {
    match path.strip_prefix(base) {
        Ok(relative) if !relative.as_os_str().is_empty() => relative.to_path_buf(),
        _ => path.to_path_buf(),
    }
}

fn nested_section_paths(meta: &'static Meta) -> Vec<Vec<&'static str>> {
    let mut paths = Vec::new();
    collect_nested_section_paths(meta, &mut Vec::new(), &mut paths);
    paths
}

fn collect_nested_section_paths(
    meta: &'static Meta,
    prefix: &mut Vec<&'static str>,
    paths: &mut Vec<Vec<&'static str>>,
) {
    for field in meta.fields {
        if let FieldKind::Nested { meta } = field.kind {
            prefix.push(field.name);
            paths.push(prefix.clone());
            collect_nested_section_paths(meta, prefix, paths);
            prefix.pop();
        }
    }
}

fn immediate_child_section_paths(
    meta: &'static Meta,
    section_path: &[&'static str],
) -> Vec<Vec<&'static str>> {
    let Some(section_meta) = meta_at_path(meta, section_path) else {
        return Vec::new();
    };

    section_meta
        .fields
        .iter()
        .filter_map(|field| match field.kind {
            FieldKind::Nested { .. } => {
                let mut path = section_path.to_vec();
                path.push(field.name);
                Some(path)
            }
            FieldKind::Leaf { .. } => None,
        })
        .collect()
}

/// Emits Figment source metadata for every leaf field at TRACE level.
///
/// This function returns immediately unless `tracing` has TRACE enabled. Callers
/// can invoke it after initializing their tracing subscriber from the loaded log
/// configuration.
///
/// # Type Parameters
///
/// - `S`: Config schema whose metadata declares the field paths to trace.
///
/// # Arguments
///
/// - `figment`: Runtime source graph used to load the config.
///
/// # Returns
///
/// This function only emits tracing events and returns no value.
pub fn trace_config_sources<S>(figment: &Figment)
where
    S: ConfigSchema,
{
    if !tracing::enabled!(tracing::Level::TRACE) {
        return;
    }

    for path in leaf_config_paths(&S::META) {
        let source = config_source_for_path(figment, &path);
        trace!(target: "rust_config_tree::config", config_key = %path, source = %source, "config source");
    }
}

fn config_source_for_path(figment: &Figment, path: &str) -> String {
    match figment.find_metadata(path) {
        Some(metadata) => render_metadata(metadata, path),
        None => "confique default or unset optional field".to_owned(),
    }
}

fn render_metadata(metadata: &Metadata, path: &str) -> String {
    match &metadata.source {
        Some(Source::File(path)) => format!("{} `{}`", metadata.name, path.display()),
        Some(Source::Custom(value)) => format!("{} `{value}`", metadata.name),
        Some(Source::Code(location)) => {
            format!("{} {}:{}", metadata.name, location.file(), location.line())
        }
        Some(_) => metadata.name.to_string(),
        None => {
            let parts = path.split('.').collect::<Vec<_>>();
            let native = metadata.interpolate(&Profile::Default, &parts);

            format!("{} `{native}`", metadata.name)
        }
    }
}

fn leaf_config_paths(meta: &'static Meta) -> Vec<String> {
    let mut paths = Vec::new();
    collect_leaf_config_paths(meta, "", &mut paths);
    paths
}

fn collect_leaf_config_paths(meta: &'static Meta, prefix: &str, paths: &mut Vec<String>) {
    for field in meta.fields {
        let path = if prefix.is_empty() {
            field.name.to_owned()
        } else {
            format!("{prefix}.{}", field.name)
        };

        match field.kind {
            FieldKind::Leaf { .. } => paths.push(path),
            FieldKind::Nested { meta } => collect_leaf_config_paths(meta, &path, paths),
        }
    }
}

fn infer_section_path_from_path<S>(path: &Path) -> Option<Vec<&'static str>>
where
    S: ConfigSchema,
{
    let path_tokens = normalized_path_tokens(path);
    let file_token = path
        .file_stem()
        .and_then(OsStr::to_str)
        .map(normalize_token)
        .unwrap_or_default();

    nested_section_paths(&S::META)
        .into_iter()
        .filter_map(|section_path| {
            let score = section_path_score(&section_path, &path_tokens, &file_token);
            (score > 0).then_some((score, section_path))
        })
        .max_by_key(|(score, section_path)| (*score, section_path.len()))
        .map(|(_, section_path)| section_path)
}

fn normalized_path_tokens(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| component.as_os_str().to_str())
        .map(|component| {
            Path::new(component)
                .file_stem()
                .and_then(OsStr::to_str)
                .unwrap_or(component)
        })
        .map(normalize_token)
        .filter(|component| !component.is_empty())
        .collect()
}

fn normalize_token(token: &str) -> String {
    token
        .chars()
        .filter_map(|character| match character {
            '-' | ' ' => Some('_'),
            '_' => Some('_'),
            character if character.is_ascii_alphanumeric() => Some(character.to_ascii_lowercase()),
            _ => None,
        })
        .collect()
}

fn section_path_score(section_path: &[&str], path_tokens: &[String], file_token: &str) -> usize {
    let section_tokens = section_path
        .iter()
        .map(|segment| normalize_token(segment))
        .collect::<Vec<_>>();

    if path_tokens.ends_with(&section_tokens) {
        return 1_000 + section_tokens.len();
    }

    let Some(last_section_token) = section_tokens.last() else {
        return 0;
    };

    if file_token == last_section_token {
        return 500 + section_tokens.len();
    }

    if file_token.starts_with(last_section_token) || last_section_token.starts_with(file_token) {
        return 100 + last_section_token.len().min(file_token.len());
    }

    0
}

fn meta_at_path(meta: &'static Meta, section_path: &[&str]) -> Option<&'static Meta> {
    let mut current_meta = meta;
    for section in section_path {
        current_meta = current_meta.fields.iter().find_map(|field| {
            if field.name != *section {
                return None;
            }

            match field.kind {
                FieldKind::Nested { meta } => Some(meta),
                FieldKind::Leaf { .. } => None,
            }
        })?;
    }

    Some(current_meta)
}

fn render_yaml_template(
    meta: &'static Meta,
    include_paths: &[PathBuf],
    section_path: &[&'static str],
    split_paths: &[Vec<&'static str>],
) -> String {
    let mut output = String::new();
    if !include_paths.is_empty() {
        output.push_str(&render_yaml_include(include_paths));
        output.push('\n');
    }

    if section_path.is_empty() {
        render_yaml_fields(
            meta,
            &mut Vec::new(),
            split_paths,
            0,
            !include_paths.is_empty(),
            &mut output,
        );
    } else {
        render_yaml_section(meta, section_path, split_paths, &mut output);
    }

    ensure_single_trailing_newline(&mut output);
    output
}

fn render_yaml_section(
    meta: &'static Meta,
    section_path: &[&'static str],
    split_paths: &[Vec<&'static str>],
    output: &mut String,
) {
    let mut current_meta = meta;
    let mut current_path = Vec::new();

    for (depth, section) in section_path.iter().enumerate() {
        write_yaml_indent(output, depth);
        output.push('#');
        output.push_str(section);
        output.push_str(":\n");
        current_path.push(*section);

        let Some(next_meta) = meta_at_path(current_meta, &[*section]) else {
            return;
        };
        current_meta = next_meta;
    }

    render_yaml_fields(
        current_meta,
        &mut current_path,
        split_paths,
        section_path.len(),
        false,
        output,
    );
}

fn render_yaml_fields(
    meta: &'static Meta,
    current_path: &mut Vec<&'static str>,
    split_paths: &[Vec<&'static str>],
    depth: usize,
    skip_include_field: bool,
    output: &mut String,
) {
    let mut emitted_anything = false;

    for field in meta.fields {
        let FieldKind::Leaf { env, kind } = field.kind else {
            continue;
        };

        if skip_include_field && current_path.is_empty() && field.name == "include" {
            continue;
        }

        if emitted_anything {
            output.push('\n');
        }
        emitted_anything = true;
        render_yaml_leaf(field.name, field.doc, env, kind, depth, output);
    }

    for field in meta.fields {
        let FieldKind::Nested { meta } = field.kind else {
            continue;
        };

        current_path.push(field.name);
        let split_exact = split_paths.iter().any(|path| path == current_path);
        let split_descendant = split_paths
            .iter()
            .any(|path| path.starts_with(current_path) && path.len() > current_path.len());

        if split_exact {
            current_path.pop();
            continue;
        }

        if emitted_anything {
            output.push('\n');
        }
        emitted_anything = true;

        for doc in field.doc {
            write_yaml_indent(output, depth);
            output.push('#');
            output.push_str(doc);
            output.push('\n');
        }
        write_yaml_indent(output, depth);
        output.push_str(field.name);
        output.push_str(":\n");

        let child_split_paths = if split_descendant { split_paths } else { &[] };
        render_yaml_fields(
            meta,
            current_path,
            child_split_paths,
            depth + 1,
            false,
            output,
        );
        current_path.pop();
    }
}

fn render_yaml_leaf(
    name: &str,
    doc: &[&str],
    env: Option<&str>,
    kind: LeafKind,
    depth: usize,
    output: &mut String,
) {
    let mut emitted_doc_comment = false;
    for doc in doc {
        write_yaml_indent(output, depth);
        output.push('#');
        output.push_str(doc);
        output.push('\n');
        emitted_doc_comment = true;
    }

    if let Some(env) = env {
        if emitted_doc_comment {
            write_yaml_indent(output, depth);
            output.push_str("#\n");
        }
        write_yaml_indent(output, depth);
        output.push_str("# Can also be specified via environment variable `");
        output.push_str(env);
        output.push_str("`.\n");
    }

    match kind {
        LeafKind::Optional => {
            write_yaml_indent(output, depth);
            output.push('#');
            output.push_str(name);
            output.push_str(":\n");
        }
        LeafKind::Required { default } => {
            write_yaml_indent(output, depth);
            match default {
                Some(default) => {
                    output.push_str("# Default value: ");
                    output.push_str(&render_yaml_expr(&default));
                    output.push('\n');
                    write_yaml_indent(output, depth);
                    output.push('#');
                    output.push_str(name);
                    output.push_str(": ");
                    output.push_str(&render_yaml_expr(&default));
                    output.push('\n');
                }
                None => {
                    output.push_str("# Required! This value must be specified.\n");
                    write_yaml_indent(output, depth);
                    output.push('#');
                    output.push_str(name);
                    output.push_str(":\n");
                }
            }
        }
    }
}

fn render_yaml_expr(expr: &Expr) -> String {
    match expr {
        Expr::Str(value) => render_plain_or_quoted_string(value),
        Expr::Float(value) => value.to_string(),
        Expr::Integer(value) => value.to_string(),
        Expr::Bool(value) => value.to_string(),
        Expr::Array(items) => {
            let items = items
                .iter()
                .map(render_yaml_expr)
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{items}]")
        }
        Expr::Map(entries) => {
            let entries = entries
                .iter()
                .map(|entry| {
                    format!(
                        "{}: {}",
                        render_yaml_map_key(&entry.key),
                        render_yaml_expr(&entry.value)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{ {entries} }}")
        }
        _ => String::new(),
    }
}

fn render_yaml_map_key(key: &MapKey) -> String {
    match key {
        MapKey::Str(value) => render_plain_or_quoted_string(value),
        MapKey::Float(value) => value.to_string(),
        MapKey::Integer(value) => value.to_string(),
        MapKey::Bool(value) => value.to_string(),
        _ => String::new(),
    }
}

fn render_plain_or_quoted_string(value: &str) -> String {
    let needs_quotes = value.is_empty()
        || value.starts_with([
            ' ', '#', '{', '}', '[', ']', ',', '&', '*', '!', '|', '>', '\'', '"',
        ])
        || value.contains([':', '\n', '\r', '\t']);

    if needs_quotes {
        quote_path(Path::new(value))
    } else {
        value.to_owned()
    }
}

fn write_yaml_indent(output: &mut String, depth: usize) {
    for _ in 0..depth {
        output.push_str("  ");
    }
}

fn ensure_single_trailing_newline(output: &mut String) {
    if output.ends_with('\n') {
        while output.ends_with("\n\n") {
            output.pop();
        }
    } else {
        output.push('\n');
    }
}

fn template_for_path_with_includes<S>(
    path: &Path,
    include_paths: &[PathBuf],
) -> ConfigResult<String>
where
    S: ConfigSchema,
{
    let template = template_for_path::<S>(path)?;
    if include_paths.is_empty() {
        return Ok(template);
    }

    let template = match ConfigFormat::from_path(path) {
        ConfigFormat::Yaml => {
            let template = strip_prefix_once(&template, "# Default value: []\n#include: []\n\n");
            format!("{}\n{template}", render_yaml_include(include_paths))
        }
        ConfigFormat::Toml => {
            let template = strip_prefix_once(&template, "# Default value: []\n#include = []\n\n");
            format!("{}\n{template}", render_toml_include(include_paths))
        }
        ConfigFormat::Json => {
            let body = template.strip_prefix("{\n").unwrap_or(&template);
            let body = strip_prefix_once(body, "  // Default value: []\n  //include: [],\n\n");
            format!("{{\n{}\n{body}", render_json5_include(include_paths))
        }
    };

    Ok(template)
}

fn render_yaml_include(paths: &[PathBuf]) -> String {
    let mut out = String::from("include:\n");
    for path in paths {
        out.push_str("  - ");
        out.push_str(&quote_path(path));
        out.push('\n');
    }
    out
}

fn render_toml_include(paths: &[PathBuf]) -> String {
    let entries = paths
        .iter()
        .map(|path| quote_path(path))
        .collect::<Vec<_>>()
        .join(", ");
    format!("include = [{entries}]\n")
}

fn render_json5_include(paths: &[PathBuf]) -> String {
    let mut out = String::from("  include: [\n");
    for path in paths {
        out.push_str("    ");
        out.push_str(&quote_path(path));
        out.push_str(",\n");
    }
    out.push_str("  ],\n");
    out
}

fn quote_path(path: &Path) -> String {
    serde_json::to_string(&path.to_string_lossy()).expect("path string serialization cannot fail")
}

fn strip_prefix_once<'a>(value: &'a str, prefix: &str) -> &'a str {
    value.strip_prefix(prefix).unwrap_or(value)
}

fn yaml_options() -> confique::yaml::FormatOptions {
    let mut options = confique::yaml::FormatOptions::default();
    options.indent = 2;
    options.general.comments = true;
    options.general.env_keys = true;
    options.general.nested_field_gap = 1;
    options
}

fn toml_options() -> confique::toml::FormatOptions {
    let mut options = confique::toml::FormatOptions::default();
    options.general.comments = true;
    options.general.env_keys = true;
    options.general.nested_field_gap = 1;
    options
}

fn json5_options() -> confique::json5::FormatOptions {
    let mut options = confique::json5::FormatOptions::default();
    options.indent = 2;
    options.general.comments = true;
    options.general.env_keys = true;
    options.general.nested_field_gap = 1;
    options
}

#[cfg(test)]
#[path = "unit_tests/config.rs"]
mod unit_tests;
