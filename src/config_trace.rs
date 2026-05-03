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
fn config_source_for_path(figment: &Figment, path: &str) -> String {
    match figment.find_metadata(path) {
        Some(metadata) => render_metadata(metadata, path),
        None => "confique default or unset optional field".to_owned(),
    }
}

/// Converts Figment metadata into a compact trace label.
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
fn leaf_config_paths(meta: &'static Meta) -> Vec<String> {
    let mut paths = Vec::new();
    collect_leaf_config_paths(meta, "", &mut paths);
    paths
}

/// Recursively appends leaf field paths from `confique` metadata.
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
