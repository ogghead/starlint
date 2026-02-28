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

use crate::rule::NativeRule;

/// Strip `JSDoc` comment markers from a single line within a `/** ... */` block.
///
/// Handles single-line `/** @tag */`, multi-line `* @tag`, and boundary lines.
pub(crate) fn trim_jsdoc_line(line: &str) -> &str {
    let trimmed = line.trim();
    let without_open = trimmed.strip_prefix("/**").unwrap_or(trimmed);
    let without_close = without_open.strip_suffix("*/").unwrap_or(without_open);
    without_close.trim_start_matches('*').trim()
}

/// Return all `JSDoc` rules.
#[must_use]
pub fn category_rules() -> Vec<Box<dyn NativeRule>> {
    vec![
        Box::new(check_access::CheckAccess),
        Box::new(check_param_names::CheckParamNames),
        Box::new(check_property_names::CheckPropertyNames),
        Box::new(check_tag_names::CheckTagNames),
        Box::new(check_types::CheckTypes),
        Box::new(check_values::CheckValues),
        Box::new(empty_tags::EmptyTags),
        Box::new(implements_on_classes::ImplementsOnClasses),
        Box::new(match_description::MatchDescription),
        Box::new(match_name::MatchName),
        Box::new(no_defaults::NoDefaults),
        Box::new(no_multi_asterisks::NoMultiAsterisks),
        Box::new(no_restricted_syntax::NoRestrictedSyntax),
        Box::new(require_description::RequireDescription),
        Box::new(require_param::RequireParam),
        Box::new(require_param_description::RequireParamDescription),
        Box::new(require_param_type::RequireParamType),
        Box::new(require_returns::RequireReturns),
    ]
}
