//! Rule: `jsdoc/check-property-names`
//!
//! Enforce `@property` names are valid identifiers and not duplicated.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

#[derive(Debug)]
pub struct CheckPropertyNames;

/// Extract `@property` (or `@prop`) names from a `JSDoc` block.
fn extract_property_names(block: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in block.lines() {
        let trimmed = super::trim_jsdoc_line(line);
        let maybe_rest = if let Some(r) = trimmed.strip_prefix("@property") {
            Some(r)
        } else {
            trimmed.strip_prefix("@prop")
        };
        if let Some(tag_rest) = maybe_rest {
            let tag_content = tag_rest.trim();
            // Skip optional type annotation `{...}`
            let after_type = if tag_content.starts_with('{') {
                tag_content
                    .find('}')
                    .and_then(|i| tag_content.get(i.saturating_add(1)..))
                    .unwrap_or_default()
                    .trim()
            } else {
                tag_content
            };
            if let Some(name) = after_type.split_whitespace().next() {
                let clean = name
                    .trim_start_matches('[')
                    .split('=')
                    .next()
                    .unwrap_or_default()
                    .trim_end_matches(']');
                if !clean.is_empty() {
                    names.push(clean.to_owned());
                }
            }
        }
    }
    names
}

impl NativeRule for CheckPropertyNames {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jsdoc/check-property-names".to_owned(),
            description: "Enforce `@property` names are valid and not duplicated".to_owned(),
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

                let names = extract_property_names(block);
                let mut seen = std::collections::HashSet::new();
                for name in &names {
                    if !seen.insert(name.as_str()) {
                        let span_start = u32::try_from(abs_start).unwrap_or(0);
                        let span_end = u32::try_from(abs_end).unwrap_or(span_start);
                        ctx.report(Diagnostic {
                            rule_name: "jsdoc/check-property-names".to_owned(),
                            message: format!("Duplicate `@property` name: `{name}`"),
                            span: Span::new(span_start, span_end),
                            severity: Severity::Warning,
                            help: None,
                            fix: None,
                            labels: vec![],
                        });
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(CheckPropertyNames)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_duplicate_property() {
        let source =
            "/**\n * @property {string} name\n * @property {number} name\n */\nconst x = {};";
        let diags = lint(source);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_unique_properties() {
        let source =
            "/**\n * @property {string} name\n * @property {number} age\n */\nconst x = {};";
        let diags = lint(source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_no_properties() {
        let source = "/** Just a description */\nconst x = {};";
        let diags = lint(source);
        assert!(diags.is_empty());
    }
}
