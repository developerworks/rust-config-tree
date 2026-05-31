//! Template normalization for transparent array split sections.

use serde_yaml::{Mapping, Value};

use crate::config_util::ensure_single_trailing_newline;

/// Normalizes one generated transparent split section YAML template.
pub fn normalize_transparent_split_template(
    content: &str,
    section: &str,
    inner_field: &str,
) -> String {
    let without_section = strip_section_root_key(content, section);
    let without_items = strip_items_wrapper(&without_section, inner_field);
    let (prefix, body) = split_template_prefix_and_body(&without_items);
    let body = rewrite_body_as_block_yaml(&body);
    let prefix = strip_default_value_comments(&prefix);

    if prefix.is_empty() {
        let mut output = body;
        ensure_single_trailing_newline(&mut output);
        return output;
    }

    let mut output = format!("{prefix}\n{body}");
    ensure_single_trailing_newline(&mut output);
    output
}

fn strip_section_root_key(content: &str, section: &str) -> String {
    let section_prefix = format!("{section}:");
    let mut lines = content.lines().collect::<Vec<_>>();
    let mut section_line = None;

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if *trimmed == section_prefix {
            section_line = Some(index);
        }
        break;
    }

    let Some(section_line) = section_line else {
        return content.to_string();
    };

    lines.remove(section_line);
    lines
        .into_iter()
        .map(|line| line.strip_prefix("  ").unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn strip_items_wrapper(content: &str, inner_field: &str) -> String {
    let items_prefix = format!("{inner_field}:");
    let mut lines = content.lines().collect::<Vec<_>>();
    let mut items_line = None;

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed == items_prefix || trimmed.starts_with(&items_prefix) {
            items_line = Some(index);
        }
        break;
    }

    let Some(items_line) = items_line else {
        return content.to_string();
    };

    let inline_value = lines[items_line]
        .trim()
        .strip_prefix(&items_prefix)
        .map(str::trim)
        .filter(|value| !value.is_empty());

    lines.remove(items_line);

    if let Some(value) = inline_value {
        lines.insert(items_line, value);
    }

    lines
        .into_iter()
        .map(|line| line.strip_prefix("  ").unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn split_template_prefix_and_body(content: &str) -> (String, String) {
    let mut prefix = Vec::new();
    let mut body = Vec::new();
    let mut seen_body = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if !seen_body && (trimmed.is_empty() || trimmed.starts_with('#')) {
            prefix.push(line);
            continue;
        }
        seen_body = true;
        body.push(line);
    }

    (prefix.join("\n"), body.join("\n"))
}

fn strip_default_value_comments(prefix: &str) -> String {
    prefix
        .lines()
        .filter(|line| !line.trim().starts_with("# Default value:"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn rewrite_body_as_block_yaml(body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return "[]".to_string();
    }

    let Ok(value) = serde_yaml::from_str::<Value>(trimmed) else {
        return body.to_string();
    };

    let pruned = prune_template_yaml_value(value);
    serde_yaml::to_string(&pruned)
        .map(|content| content.trim().to_string())
        .unwrap_or_else(|_| body.to_string())
}

fn prune_template_yaml_value(value: Value) -> Value {
    match value {
        Value::Mapping(map) => {
            let mut pruned = Mapping::new();
            for (key, child) in map {
                let next = prune_template_yaml_value(child);
                if should_omit_template_yaml_value(&next) {
                    continue;
                }
                pruned.insert(key, next);
            }
            Value::Mapping(pruned)
        }
        Value::Sequence(items) => Value::Sequence(
            items
                .into_iter()
                .map(prune_template_yaml_value)
                .filter(|item| !should_omit_template_yaml_value(item))
                .collect(),
        ),
        other => other,
    }
}

fn should_omit_template_yaml_value(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::Sequence(items) => items.is_empty(),
        Value::Mapping(map) => map.is_empty(),
        _ => false,
    }
}

/// Returns whether one section path should render as a body-only transparent array.
pub fn is_transparent_split_section(
    section_path: &[&str],
    transparent_paths: &[Vec<&str>],
) -> bool {
    transparent_paths
        .iter()
        .any(|path| path.as_slice() == section_path)
}
