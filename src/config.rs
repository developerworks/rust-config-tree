//! High-level `confique` integration and config-template rendering.
//!
//! This module loads `.env` values, loads recursive config trees into a final
//! `confique` schema, and renders example templates that mirror the same
//! include tree. YAML templates can also be split across nested schema sections.

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use confique::{
    Config, File, FileFormat,
    meta::{Expr, FieldKind, LeafKind, MapKey, Meta},
};

use crate::{
    ConfigError, ConfigSource, ConfigTreeOptions, IncludeOrder, absolutize_lexical,
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

/// Loads a complete `confique` schema from a root config path.
///
/// The loader follows recursive include paths exposed by [`ConfigSchema`],
/// resolves relative include paths from the declaring file, detects include
/// cycles, loads the first `.env` file found from the root config directory
/// upward, and then asks `confique` to merge the collected layers with
/// environment values. Existing process environment variables take precedence
/// over values loaded from `.env`.
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
    let path = path.as_ref();
    load_dotenv_for_path(path)?;

    let mut builder = S::builder().env();
    let tree = ConfigTreeOptions::default()
        .include_order(IncludeOrder::Reverse)
        .load(
            path,
            |path| -> ConfigResult<ConfigSource<<S as Config>::Layer>> {
                let layer = load_layer::<S>(path)?;
                let include_paths = S::include_paths(&layer);
                Ok(ConfigSource::new(layer, include_paths))
            },
        )?;

    for layer in tree.into_values() {
        builder = builder.preloaded(layer);
    }

    Ok(builder.load()?)
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
    Ok(File::with_format(path, ConfigFormat::from_path(path).as_file_format()).load()?)
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
            let mut include_paths = if node_source_path.exists() {
                let layer = load_layer::<S>(node_source_path)?;
                S::include_paths(&layer)
            } else {
                Vec::new()
            };

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
