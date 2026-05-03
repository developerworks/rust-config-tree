//! Runtime source tracing for loaded config values.
//!
//! The loader can emit TRACE events showing whether each leaf field came from a
//! file, an environment variable, or a `confique` default. This stays separate
//! from loading so normal users pay only the `tracing::enabled!` check.

use confique::meta::{FieldKind, Meta};
use figment::{Figment, Metadata, Profile, Source};
use tracing::trace;

use crate::config::ConfigSchema;

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
///
/// # Examples
///
/// ```
/// use confique::Config;
/// use figment::Figment;
/// use rust_config_tree::{ConfigSchema, trace_config_sources};
///
/// #[derive(Config)]
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
/// trace_config_sources::<AppConfig>(&Figment::new());
/// ```
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

/// Renders the runtime source for one schema field path.
///
/// # Arguments
///
/// - `figment`: Runtime source graph used to locate metadata.
/// - `path`: Dot-separated schema field path.
///
/// # Returns
///
/// Returns a human-readable source label for the field path.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn config_source_for_path(figment: &Figment, path: &str) -> String {
    match figment.find_metadata(path) {
        Some(metadata) => render_metadata(metadata, path),
        None => "confique default or unset optional field".to_owned(),
    }
}

/// Converts Figment metadata into a compact trace label.
///
/// # Arguments
///
/// - `metadata`: Figment metadata returned for a field path.
/// - `path`: Dot-separated schema field path used for metadata interpolation.
///
/// # Returns
///
/// Returns a compact label describing the provider and native source.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
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

/// Collects dot-separated field paths for every leaf config value.
///
/// # Arguments
///
/// - `meta`: Root `confique` metadata to traverse.
///
/// # Returns
///
/// Returns dot-separated field paths for every leaf config value.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn leaf_config_paths(meta: &'static Meta) -> Vec<String> {
    let mut paths = Vec::new();
    collect_leaf_config_paths(meta, "", &mut paths);
    paths
}

/// Recursively appends leaf field paths from `confique` metadata.
///
/// # Arguments
///
/// - `meta`: Current `confique` metadata node.
/// - `prefix`: Dot-separated field path prefix for `meta`.
/// - `paths`: Output list receiving leaf field paths.
///
/// # Returns
///
/// Returns no value; `paths` is updated directly.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
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
