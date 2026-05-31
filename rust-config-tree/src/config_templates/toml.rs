//! TOML rendering for configuration templates.

use std::path::PathBuf;

use confique::meta::{Expr, FieldKind, LeafKind, MapKey, Meta};

use super::{
    fields::{has_renderable_fields, is_env_only_field},
    render::{quote_path, render_toml_include},
    section::meta_at_path,
};
use crate::config_util::ensure_single_trailing_newline;

/// Renders a TOML template for either the root config or one split section.
pub(super) fn render_toml_template(
    meta: &'static Meta,
    include_paths: &[PathBuf],
    section_path: &[&'static str],
    split_paths: &[Vec<&'static str>],
    env_only_paths: &[Vec<&'static str>],
) -> String {
    let mut output = String::new();

    if section_path.is_empty() {
        if !include_paths.is_empty() {
            output.push_str(&render_toml_include(include_paths));
            output.push('\n');
        }
        render_toml_fields(
            meta,
            &mut Vec::new(),
            split_paths,
            env_only_paths,
            &[],
            !include_paths.is_empty(),
            &mut output,
        );
    } else {
        render_toml_section(
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

fn render_toml_section(
    meta: &'static Meta,
    section_path: &[&'static str],
    include_paths: &[PathBuf],
    split_paths: &[Vec<&'static str>],
    env_only_paths: &[Vec<&'static str>],
    output: &mut String,
) {
    let mut current_meta = meta;
    let mut current_path = Vec::new();
    let mut table_prefix = Vec::new();

    for (depth, section) in section_path.iter().enumerate() {
        let is_target_section = depth + 1 == section_path.len();
        if !is_target_section {
            write_toml_comment(output, &format!("[{}]", section));
        }
        table_prefix.push(*section);
        current_path.push(*section);

        let Some(next_meta) = meta_at_path(current_meta, &[*section]) else {
            return;
        };
        current_meta = next_meta;
    }

    if !include_paths.is_empty() {
        append_toml_include(output, include_paths, &table_prefix);
        output.push('\n');
    }

    render_toml_fields(
        current_meta,
        &mut current_path,
        split_paths,
        env_only_paths,
        &table_prefix,
        false,
        output,
    );
}

fn render_toml_fields(
    meta: &'static Meta,
    current_path: &mut Vec<&'static str>,
    split_paths: &[Vec<&'static str>],
    env_only_paths: &[Vec<&'static str>],
    table_prefix: &[&'static str],
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
        render_toml_leaf(field.name, field.doc, env, kind, output);
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
            write_toml_comment(output, doc);
        }

        let mut child_table = table_prefix.to_vec();
        child_table.push(field.name);
        write_toml_table_header(output, &child_table);
        output.push('\n');

        render_toml_fields(
            meta,
            current_path,
            child_split_paths,
            env_only_paths,
            &child_table,
            false,
            output,
        );
        current_path.pop();
    }
}

fn render_toml_leaf(
    name: &str,
    doc: &[&str],
    env: Option<&str>,
    kind: LeafKind,
    output: &mut String,
) {
    let mut emitted_doc_comment = false;
    for doc in doc {
        write_toml_comment(output, doc);
        emitted_doc_comment = true;
    }

    if let Some(env) = env {
        if emitted_doc_comment {
            write_toml_comment(output, "");
        }
        write_toml_comment(
            output,
            &format!(" Can also be specified via environment variable `{env}`."),
        );
    }

    match kind {
        LeafKind::Optional => {
            write_toml_comment(output, &format!("{name} ="));
        }
        LeafKind::Required { default } => match default {
            Some(default) => {
                write_toml_comment(
                    output,
                    &format!(" Default value: {}", render_toml_expr(&default)),
                );
                output.push_str(name);
                output.push_str(" = ");
                output.push_str(&render_toml_expr(&default));
                output.push('\n');
            }
            None => {
                write_toml_comment(output, " Required! This value must be specified.");
                write_toml_comment(output, &format!("{name} ="));
            }
        },
    }
}

fn render_toml_expr(expr: &Expr) -> String {
    match expr {
        Expr::Str(value) => quote_path(std::path::Path::new(value)),
        Expr::Float(value) => value.to_string(),
        Expr::Integer(value) => value.to_string(),
        Expr::Bool(value) => value.to_string(),
        Expr::Array(items) => {
            let items = items
                .iter()
                .map(render_toml_expr)
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{items}]")
        }
        Expr::Map(entries) => {
            let entries = entries
                .iter()
                .map(|entry| {
                    format!(
                        "{} = {}",
                        render_toml_map_key(&entry.key),
                        render_toml_expr(&entry.value)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{ {entries} }}")
        }
        _ => String::new(),
    }
}

fn render_toml_map_key(key: &MapKey) -> String {
    match key {
        MapKey::Str(value) if is_valid_bare_key(value) => (*value).to_string(),
        MapKey::Str(value) => quote_path(std::path::Path::new(value)),
        MapKey::Float(value) => value.to_string(),
        MapKey::Integer(value) => value.to_string(),
        MapKey::Bool(value) => value.to_string(),
        _ => String::new(),
    }
}

fn is_valid_bare_key(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn write_toml_comment(output: &mut String, comment: &str) {
    output.push('#');
    output.push_str(comment);
    output.push('\n');
}

fn write_toml_table_header(output: &mut String, table_path: &[&str]) {
    output.push('[');
    output.push_str(&table_path.join("."));
    output.push(']');
    output.push('\n');
}

fn append_toml_include(output: &mut String, paths: &[PathBuf], table_prefix: &[&str]) {
    if table_prefix.is_empty() {
        output.push_str(&render_toml_include(paths));
        return;
    }

    write_toml_table_header(output, table_prefix);
    let entries = paths
        .iter()
        .map(|path| quote_path(path.as_path()))
        .collect::<Vec<_>>()
        .join(", ");
    output.push_str("include = [");
    output.push_str(&entries);
    output.push_str("]\n");
}
