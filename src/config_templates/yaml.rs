//! YAML rendering for split nested configuration sections.

use std::path::{Path, PathBuf};

use confique::meta::{Expr, FieldKind, LeafKind, MapKey, Meta};

use super::{
    render::{quote_path, render_yaml_include},
    section::meta_at_path,
};
use crate::config_util::ensure_single_trailing_newline;

/// Renders a YAML template for either the root config or one split section.
///
/// # Arguments
///
/// - `meta`: Root config metadata used to render fields.
/// - `include_paths`: Include paths to emit at the top of the template.
/// - `section_path`: Empty for the root template, or the split section path.
/// - `split_paths`: Section paths split into separate templates.
/// - `env_only_paths`: Leaf field paths omitted from generated config files.
///
/// # Returns
///
/// Returns complete YAML template content.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
pub(super) fn render_yaml_template(
    meta: &'static Meta,
    include_paths: &[PathBuf],
    section_path: &[&'static str],
    split_paths: &[Vec<&'static str>],
    env_only_paths: &[Vec<&'static str>],
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
            env_only_paths,
            0,
            !include_paths.is_empty(),
            &mut output,
        );
    } else {
        render_yaml_section(meta, section_path, split_paths, env_only_paths, &mut output);
    }

    ensure_single_trailing_newline(&mut output);
    output
}

/// Renders the commented ancestor context for a split YAML section template.
///
/// # Arguments
///
/// - `meta`: Root config metadata used to find the section metadata.
/// - `section_path`: Split section path represented by the template.
/// - `split_paths`: Section paths split into separate templates.
/// - `env_only_paths`: Leaf field paths omitted from generated config files.
/// - `output`: Output string receiving rendered YAML.
///
/// # Returns
///
/// Returns no value; `output` is updated directly.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn render_yaml_section(
    meta: &'static Meta,
    section_path: &[&'static str],
    split_paths: &[Vec<&'static str>],
    env_only_paths: &[Vec<&'static str>],
    output: &mut String,
) {
    let mut current_meta = meta;
    let mut current_path = Vec::new();

    for (depth, section) in section_path.iter().enumerate() {
        // Section templates remain valid partial YAML. Ancestor section keys are
        // emitted as comments so editors show context without duplicating data.
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
        env_only_paths,
        section_path.len(),
        false,
        output,
    );
}

/// Renders leaf fields and non-split nested sections for one metadata node.
///
/// # Arguments
///
/// - `meta`: Metadata node whose fields should be rendered.
/// - `current_path`: Mutable section path for `meta`.
/// - `split_paths`: Section paths split into separate templates.
/// - `env_only_paths`: Leaf field paths omitted from generated config files.
/// - `depth`: YAML indentation depth.
/// - `skip_include_field`: Whether to omit the root include field.
/// - `output`: Output string receiving rendered YAML.
///
/// # Returns
///
/// Returns no value; `output` and `current_path` are updated during traversal.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn render_yaml_fields(
    meta: &'static Meta,
    current_path: &mut Vec<&'static str>,
    split_paths: &[Vec<&'static str>],
    env_only_paths: &[Vec<&'static str>],
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

        if is_env_only_field(current_path, field.name, env_only_paths) {
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

        let child_split_paths = if split_descendant { split_paths } else { &[] };
        if !has_renderable_yaml_fields(meta, current_path, child_split_paths, env_only_paths, false)
        {
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
        output.push('#');
        output.push_str(field.name);
        output.push_str(":\n");

        render_yaml_fields(
            meta,
            current_path,
            child_split_paths,
            env_only_paths,
            depth + 1,
            false,
            output,
        );
        current_path.pop();
    }
}

/// Returns whether `meta` has any fields that should be rendered.
///
/// # Arguments
///
/// - `meta`: Metadata node to inspect.
/// - `current_path`: Section path for `meta`.
/// - `split_paths`: Section paths split into separate templates.
/// - `env_only_paths`: Leaf field paths omitted from generated config files.
/// - `skip_include_field`: Whether to omit the root include field.
///
/// # Returns
///
/// Returns `true` when at least one leaf or nested section is renderable.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn has_renderable_yaml_fields(
    meta: &'static Meta,
    current_path: &[&'static str],
    split_paths: &[Vec<&'static str>],
    env_only_paths: &[Vec<&'static str>],
    skip_include_field: bool,
) -> bool {
    for field in meta.fields {
        let FieldKind::Leaf { .. } = field.kind else {
            continue;
        };

        if skip_include_field && current_path.is_empty() && field.name == "include" {
            continue;
        }

        if !is_env_only_field(current_path, field.name, env_only_paths) {
            return true;
        }
    }

    for field in meta.fields {
        let FieldKind::Nested { meta } = field.kind else {
            continue;
        };

        let mut child_path = current_path.to_vec();
        child_path.push(field.name);

        let split_exact = split_paths.iter().any(|path| path == &child_path);
        if split_exact {
            continue;
        }

        let split_descendant = split_paths
            .iter()
            .any(|path| path.starts_with(&child_path) && path.len() > child_path.len());
        let child_split_paths = if split_descendant { split_paths } else { &[] };

        if has_renderable_yaml_fields(meta, &child_path, child_split_paths, env_only_paths, false) {
            return true;
        }
    }

    false
}

/// Returns whether a leaf field path is marked env-only.
///
/// # Arguments
///
/// - `current_path`: Section path for the field.
/// - `field_name`: Leaf field name.
/// - `env_only_paths`: Leaf field paths omitted from generated config files.
///
/// # Returns
///
/// Returns `true` when the full field path is in `env_only_paths`.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn is_env_only_field(
    current_path: &[&'static str],
    field_name: &'static str,
    env_only_paths: &[Vec<&'static str>],
) -> bool {
    env_only_paths.iter().any(|path| {
        path.len() == current_path.len() + 1
            && path.starts_with(current_path)
            && path.last() == Some(&field_name)
    })
}

/// Renders one leaf field with docs, environment hint, and default value.
///
/// # Arguments
///
/// - `name`: Config field name.
/// - `doc`: Documentation lines emitted for the field.
/// - `env`: Optional environment variable name for the field.
/// - `kind`: Leaf field kind and default metadata.
/// - `depth`: YAML indentation depth.
/// - `output`: Output string receiving rendered YAML.
///
/// # Returns
///
/// Returns no value; `output` is updated directly.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
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

/// Renders a `confique` default expression as inline YAML.
///
/// # Arguments
///
/// - `expr`: Default expression from `confique` metadata.
///
/// # Returns
///
/// Returns an inline YAML representation of the expression.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
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

/// Renders a map key in the inline YAML representation.
///
/// # Arguments
///
/// - `key`: Map key expression from `confique` metadata.
///
/// # Returns
///
/// Returns an inline YAML representation of the map key.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn render_yaml_map_key(key: &MapKey) -> String {
    match key {
        MapKey::Str(value) => render_plain_or_quoted_string(value),
        MapKey::Float(value) => value.to_string(),
        MapKey::Integer(value) => value.to_string(),
        MapKey::Bool(value) => value.to_string(),
        _ => String::new(),
    }
}

/// Renders simple YAML scalars plainly and quotes ambiguous strings.
///
/// # Arguments
///
/// - `value`: String scalar to render.
///
/// # Returns
///
/// Returns `value` unchanged when plain YAML is safe, otherwise quoted.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
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

/// Writes two-space indentation for a YAML nesting depth.
///
/// # Arguments
///
/// - `output`: Output string receiving indentation.
/// - `depth`: Number of two-space indentation levels to write.
///
/// # Returns
///
/// Returns no value; `output` is updated directly.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn write_yaml_indent(output: &mut String, depth: usize) {
    for _ in 0..depth {
        output.push_str("  ");
    }
}
