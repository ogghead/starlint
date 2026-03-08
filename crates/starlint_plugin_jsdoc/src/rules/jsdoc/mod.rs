//! `JSDoc` lint rules.
//!
//! Rules are prefixed with `jsdoc/` in config and output.

pub mod check_access;
pub mod check_param_names;
pub mod check_property_names;
pub mod check_tag_names;
pub mod check_types;
pub mod check_values;
pub mod empty_tags;
pub mod implements_on_classes;
pub mod match_description;
pub mod match_name;
pub mod no_defaults;
pub mod no_multi_asterisks;
pub mod no_restricted_syntax;
pub mod require_description;
pub mod require_param;
pub mod require_param_description;
pub mod require_param_type;
pub mod require_returns;

/// Strip `JSDoc` comment markers from a single line within a `/** ... */` block.
///
/// Handles single-line `/** @tag */`, multi-line `* @tag`, and boundary lines.
pub(crate) fn trim_jsdoc_line(line: &str) -> &str {
    let trimmed = line.trim();
    let without_open = trimmed.strip_prefix("/**").unwrap_or(trimmed);
    let without_close = without_open.strip_suffix("*/").unwrap_or(without_open);
    without_close.trim_start_matches('*').trim()
}
