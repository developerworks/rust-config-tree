//! JSON5 rendering for configuration templates.

use std::path::PathBuf;

use confique::meta::{Expr, FieldKind, LeafKind, MapKey, Meta};

use super::{
    fields::{has_renderable_fields, is_env_only_field},
    render::{quote_path, render_json5_include},
    section::meta_at_path,
};
use crate::config_util::ensure_single_trailing_newline;

/// Renders a JSON5 template for either the root config or one split section.
pub(super) fn render_json5_template(
    meta: &'static Meta,
    include_paths: &[PathBuf],
    section_path: &[&'static str],
    split_paths: &[Vec<&'static str>],
    env_only_paths: &[Vec<&'static str>],
) -> String {
    let mut output = String::new();

    if section_path.is_empty() {
        output.push_str("{\n");
        if !include_paths.is_empty() {
            output.push_str(&render_json5_include(include_paths));
            output.push('\n');
        }
        render_json5_fields(
            meta,
            &mut Vec::new(),
            split_paths,
            env_only_paths,
            1,
            !include_paths.is_empty(),
            &mut output,
        );
        output.push_str("}\n");
    } else {
        render_json5_section(
            meta,
            section_path,
            include_paths,
            split_paths,
            env_only_paths,
            &mut output,
        );
    }

    ensure_single_trailing_newline(&mut output);
    output
}

fn render_json5_section(
    meta: &'static Meta,
    section_path: &[&'static str],
    include_paths: &[PathBuf],
    split_paths: &[Vec<&'static str>],
    env_only_paths: &[Vec<&'static str>],
    output: &mut String,
) {
    let mut current_meta = meta;
    let mut current_path = Vec::new();

    output.push_str("{\n");

    for (depth, section) in section_path.iter().enumerate() {
        let is_target_section = depth + 1 == section_path.len();
        if is_target_section {
            write_json5_indent(output, depth + 1);
            output.push_str(section);
            output.push_str(": {\n");
        } else {
            write_json5_indent(output, depth + 1);
            output.push_str("// ");
            output.push_str(section);
            output.push_str(": {\n");
        }
        current_path.push(*section);

        let Some(next_meta) = meta_at_path(current_meta, &[*section]) else {
            return;
        };
        current_meta = next_meta;
    }

    let depth = section_path.len() + 1;

    if !include_paths.is_empty() {
        append_json5_include(output, include_paths, depth);
        output.push('\n');
    }

    render_json5_fields(
        current_meta,
        &mut current_path,
        split_paths,
        env_only_paths,
        depth,
        false,
        output,
    );

    for depth in (1..=section_path.len()).rev() {
        write_json5_indent(output, depth);
        output.push_str("},\n");
    }

    output.push_str("}\n");
}

fn render_json5_fields(
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
        render_json5_leaf(field.name, field.doc, env, kind, depth, output);
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
        if !has_renderable_fields(meta, current_path, child_split_paths, env_only_paths, false) {
            current_path.pop();
            continue;
        }

        if emitted_anything {
            output.push('\n');
        }
        emitted_anything = true;

        for doc in field.doc {
            write_json5_indent(output, depth);
            output.push_str("//");
            output.push_str(doc);
            output.push('\n');
        }
        write_json5_indent(output, depth);
        output.push_str(field.name);
        output.push_str(": {\n");

        render_json5_fields(
            meta,
            current_path,
            child_split_paths,
            env_only_paths,
            depth + 1,
            false,
            output,
        );

        write_json5_indent(output, depth);
        output.push_str("},\n");
        current_path.pop();
    }
}

fn render_json5_leaf(
    name: &str,
    doc: &[&str],
    env: Option<&str>,
    kind: LeafKind,
    depth: usize,
    output: &mut String,
) {
    let mut emitted_doc_comment = false;
    for doc in doc {
        write_json5_indent(output, depth);
        output.push_str("//");
        output.push_str(doc);
        output.push('\n');
        emitted_doc_comment = true;
    }

    if let Some(env) = env {
        if emitted_doc_comment {
            write_json5_indent(output, depth);
            output.push_str("//\n");
        }
        write_json5_indent(output, depth);
        output.push_str("// Can also be specified via environment variable `");
        output.push_str(env);
        output.push_str("`.\n");
    }

    match kind {
        LeafKind::Optional => {
            write_json5_indent(output, depth);
            output.push_str("//");
            output.push_str(name);
            output.push_str(": ,\n");
        }
        LeafKind::Required { default } => {
            write_json5_indent(output, depth);
            match default {
                Some(default) => {
                    output.push_str("// Default value: ");
                    output.push_str(&render_json5_expr(&default));
                    output.push('\n');
                    write_json5_indent(output, depth);
                    output.push_str(name);
                    output.push_str(": ");
                    output.push_str(&render_json5_expr(&default));
                    output.push_str(",\n");
                }
                None => {
                    output.push_str("// Required! This value must be specified.\n");
                    write_json5_indent(output, depth);
                    output.push_str(name);
                    output.push_str(": ,\n");
                }
            }
        }
    }
}

fn render_json5_expr(expr: &Expr) -> String {
    match expr {
        Expr::Str(value) => quote_path(std::path::Path::new(value)),
        Expr::Float(value) => value.to_string(),
        Expr::Integer(value) => value.to_string(),
        Expr::Bool(value) => value.to_string(),
        Expr::Array(items) => {
            let items = items
                .iter()
                .map(render_json5_expr)
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
                        render_json5_map_key(&entry.key),
                        render_json5_expr(&entry.value)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{ {entries} }}")
        }
        _ => String::new(),
    }
}

fn render_json5_map_key(key: &MapKey) -> String {
    match key {
        MapKey::Str(value) => quote_path(std::path::Path::new(value)),
        MapKey::Float(value) => value.to_string(),
        MapKey::Integer(value) => value.to_string(),
        MapKey::Bool(value) => value.to_string(),
        _ => String::new(),
    }
}

fn write_json5_indent(output: &mut String, depth: usize) {
    for _ in 0..depth {
        output.push_str("  ");
    }
}

fn append_json5_include(output: &mut String, paths: &[PathBuf], depth: usize) {
    write_json5_indent(output, depth);
    output.push_str("include: [\n");
    for path in paths {
        write_json5_indent(output, depth + 1);
        output.push_str(&quote_path(path));
        output.push_str(",\n");
    }
    write_json5_indent(output, depth);
    output.push_str("],\n");
}
