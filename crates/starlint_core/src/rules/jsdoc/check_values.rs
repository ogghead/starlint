//! Rule: `jsdoc/check-values`
//!
//! Enforce valid `@version`, `@since`, and `@license` values.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

#[derive(Debug)]
pub struct CheckValues;

/// Check if a string looks like a semver version (simplified).
fn is_semver_like(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return false;
    }
    parts.iter().all(|p| {
        // Allow pre-release suffix on last part (e.g. "0-beta")
        let numeric = p.split('-').next().unwrap_or_default();
        !numeric.is_empty() && numeric.chars().all(|c| c.is_ascii_digit())
    })
}

/// Known SPDX license identifiers (subset of common ones).
const COMMON_LICENSES: &[&str] = &[
    "MIT",
    "Apache-2.0",
    "GPL-2.0",
    "GPL-3.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "MPL-2.0",
    "LGPL-2.1",
    "LGPL-3.0",
    "AGPL-3.0",
    "Unlicense",
    "CC0-1.0",
    "0BSD",
];

impl NativeRule for CheckValues {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jsdoc/check-values".to_owned(),
            description: "Enforce valid `@version`, `@since`, and `@license` values".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text().to_owned();

        let mut pos = 0;
        while let Some(start) = source.get(pos..).and_then(|s| s.find("/**")) {
            let abs_start = pos.saturating_add(start);
            let search_from = abs_start.saturating_add(3);
            if let Some(end) = source.get(search_from..).and_then(|s| s.find("*/")) {
                let abs_end = search_from.saturating_add(end).saturating_add(2);
                let block = source.get(abs_start..abs_end).unwrap_or_default();

                let span_start = u32::try_from(abs_start).unwrap_or(0);
                let span_end = u32::try_from(abs_end).unwrap_or(span_start);

                for line in block.lines() {
                    let trimmed = super::trim_jsdoc_line(line);

                    // Check @version and @since for semver-like values
                    for tag in &["@version", "@since"] {
                        if let Some(rest) = trimmed.strip_prefix(tag) {
                            let value = rest.trim();
                            if !value.is_empty() && !is_semver_like(value) {
                                ctx.report(Diagnostic {
                                    rule_name: "jsdoc/check-values".to_owned(),
                                    message: format!(
                                        "Invalid `{tag}` value: `{value}`. Expected a semver-like version"
                                    ),
                                    span: Span::new(span_start, span_end),
                                    severity: Severity::Warning,
                                    help: None,
                                    fix: None,
                                    labels: vec![],
                                });
                            }
                        }
                    }

                    // Check @license for known SPDX identifiers
                    if let Some(rest) = trimmed.strip_prefix("@license") {
                        let value = rest.trim();
                        if !value.is_empty() && !COMMON_LICENSES.contains(&value) {
                            ctx.report(Diagnostic {
                                rule_name: "jsdoc/check-values".to_owned(),
                                message: format!(
                                    "Unknown `@license` value: `{value}`. Expected a known SPDX identifier"
                                ),
                                span: Span::new(span_start, span_end),
                                severity: Severity::Warning,
                                help: None,
                                fix: None,
                                labels: vec![],
                            });
                        }
                    }
                }

                pos = abs_end;
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(CheckValues)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_invalid_version() {
        let source = "/** @version notaversion */\nconst x = 1;";
        let diags = lint(source);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_valid_version() {
        let source = "/** @version 1.2.3 */\nconst x = 1;";
        let diags = lint(source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_valid_license() {
        let source = "/** @license MIT */\nconst x = 1;";
        let diags = lint(source);
        assert!(diags.is_empty());
    }
}
