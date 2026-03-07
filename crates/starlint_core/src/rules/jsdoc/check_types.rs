//! Rule: `jsdoc/check-types`
//!
//! Enforce consistent type format in `JSDoc` (e.g. `object` not `Object`).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

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

impl LintRule for CheckTypes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jsdoc/check-types".to_owned(),
            description: "Enforce consistent type format in JSDoc".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
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
                                ctx.report(Diagnostic {
                                    rule_name: "jsdoc/check-types".to_owned(),
                                    message: format!(
                                        "Use `{correct}` instead of `{wrong}` in JSDoc type"
                                    ),
                                    span: Span::new(span_start, span_end),
                                    severity: Severity::Warning,
                                    help: None,
                                    fix: None,
                                    labels: vec![],
                                });
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
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(CheckTypes)];
        lint_source(source, "test.js", &rules)
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
