//! Rule: `no-multi-str`
//!
//! Disallow multiline strings created with `\` at the end of a line.
//! Use template literals or string concatenation instead.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags multiline strings using backslash continuation.
#[derive(Debug)]
pub struct NoMultiStr;

impl LintRule for NoMultiStr {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-multi-str".to_owned(),
            description: "Disallow multiline strings".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::StringLiteral])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::StringLiteral(lit) = node else {
            return;
        };

        // Check the raw source text of the string for backslash-newline continuation
        let source = ctx.source_text();
        let raw_text = source
            .get(lit.span.start as usize..lit.span.end as usize)
            .unwrap_or("");

        // A multiline string has a backslash immediately before a newline
        let has_continuation = raw_text.contains("\\\n") || raw_text.contains("\\\r\n");

        if has_continuation {
            // Fix: convert to template literal — replace quotes with backticks
            // and remove backslash-newline continuations
            let fix = {
                let mut converted = raw_text.to_owned();
                // Remove backslash-newline continuations
                converted = converted.replace("\\\r\n", "\n");
                converted = converted.replace("\\\n", "\n");
                // Replace outer quotes with backticks
                if converted.starts_with('\'') || converted.starts_with('"') {
                    converted.replace_range(..1, "`");
                }
                if converted.ends_with('\'') || converted.ends_with('"') {
                    let last = converted.len().saturating_sub(1);
                    converted.replace_range(last.., "`");
                }
                Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: "Convert to template literal".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(lit.span.start, lit.span.end),
                        replacement: converted,
                    }],
                    is_snippet: false,
                })
            };

            ctx.report(Diagnostic {
                rule_name: "no-multi-str".to_owned(),
                message: "Multiline strings using `\\` are not recommended".to_owned(),
                span: Span::new(lit.span.start, lit.span.end),
                severity: Severity::Warning,
                help: None,
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoMultiStr)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_multiline_string() {
        let diags = lint("var x = 'hello \\\nworld';");
        assert_eq!(
            diags.len(),
            1,
            "multiline string with backslash continuation should be flagged"
        );
    }

    #[test]
    fn test_allows_single_line_string() {
        let diags = lint("var x = 'hello world';");
        assert!(diags.is_empty(), "single line string should not be flagged");
    }

    #[test]
    fn test_allows_template_literal() {
        let diags = lint("var x = `hello\nworld`;");
        assert!(diags.is_empty(), "template literal should not be flagged");
    }
}
