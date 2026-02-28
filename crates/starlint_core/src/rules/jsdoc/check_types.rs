//! Rule: `jsdoc/check-types`
//!
//! Enforce consistent type format in `JSDoc` (e.g. `object` not `Object`).

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Mapping from incorrect casing to preferred casing.
const TYPE_CORRECTIONS: &[(&str, &str)] = &[
    ("Object", "object"),
    ("Boolean", "boolean"),
    ("Number", "number"),
    ("String", "string"),
    ("Symbol", "symbol"),
    ("BigInt", "bigint"),
    ("Undefined", "undefined"),
    ("Null", "null"),
    ("Void", "void"),
];

#[derive(Debug)]
pub struct CheckTypes;

impl NativeRule for CheckTypes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jsdoc/check-types".to_owned(),
            description: "Enforce consistent type format in JSDoc".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
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

                // Find type annotations in `{...}` within JSDoc
                let mut type_pos = 0;
                while let Some(brace_start) = block.get(type_pos..).and_then(|s| s.find('{')) {
                    let abs_brace = type_pos.saturating_add(brace_start);
                    if let Some(brace_end) = block.get(abs_brace..).and_then(|s| s.find('}')) {
                        let type_str = block
                            .get(abs_brace.saturating_add(1)..abs_brace.saturating_add(brace_end))
                            .unwrap_or_default();

                        for (wrong, correct) in TYPE_CORRECTIONS {
                            if type_str
                                .split(|c: char| !c.is_alphanumeric())
                                .any(|word| word == *wrong)
                            {
                                let span_start = u32::try_from(abs_start).unwrap_or(0);
                                let span_end = u32::try_from(abs_end).unwrap_or(span_start);
                                ctx.report_warning(
                                    "jsdoc/check-types",
                                    &format!("Use `{correct}` instead of `{wrong}` in JSDoc type"),
                                    Span::new(span_start, span_end),
                                );
                            }
                        }

                        type_pos = abs_brace.saturating_add(brace_end).saturating_add(1);
                    } else {
                        break;
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(CheckTypes)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_uppercase_object() {
        let source = "/** @param {Object} x */\nfunction foo(x) {}";
        let diags = lint(source);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_lowercase_types() {
        let source = "/** @param {object} x */\nfunction foo(x) {}";
        let diags = lint(source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_flags_uppercase_string() {
        let source = "/** @returns {String} */\nfunction foo() { return ''; }";
        let diags = lint(source);
        assert_eq!(diags.len(), 1);
    }
}
