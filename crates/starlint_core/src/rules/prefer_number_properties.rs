//! Rule: `prefer-number-properties`
//!
//! Prefer `Number` static methods over global equivalents.
//! Flag `isNaN()`, `isFinite()`, `parseInt()`, `parseFloat()` as globals.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Global functions that should use `Number.*` equivalents.
const GLOBAL_FUNCTIONS: &[(&str, &str)] = &[
    ("isNaN", "Number.isNaN"),
    ("isFinite", "Number.isFinite"),
    ("parseInt", "Number.parseInt"),
    ("parseFloat", "Number.parseFloat"),
];

/// Flags global `isNaN`, `isFinite`, `parseInt`, `parseFloat` calls.
#[derive(Debug)]
pub struct PreferNumberProperties;

impl LintRule for PreferNumberProperties {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-number-properties".to_owned(),
            description: "Prefer `Number` static methods over global equivalents".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Resolve callee NodeId and extract identifier info before mutable borrow
        let Some(AstNode::IdentifierReference(id)) = ctx.node(call.callee) else {
            return;
        };

        let name = id.name.clone();
        let id_span = Span::new(id.span.start, id.span.end);

        let Some((_, replacement)) = GLOBAL_FUNCTIONS
            .iter()
            .find(|(global, _)| *global == name.as_str())
        else {
            return;
        };

        ctx.report(Diagnostic {
            rule_name: "prefer-number-properties".to_owned(),
            message: format!("Use `{replacement}()` instead of `{name}()`"),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some(format!("Replace `{name}` with `{replacement}`")),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: format!("Replace `{name}` with `{replacement}`"),
                edits: vec![Edit {
                    span: id_span,
                    replacement: (*replacement).to_owned(),
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferNumberProperties)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_global_is_nan() {
        let diags = lint("isNaN(x);");
        assert_eq!(diags.len(), 1, "should flag isNaN()");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("Number.isNaN"),
            "fix should replace with Number.isNaN"
        );
    }

    #[test]
    fn test_flags_global_parse_int() {
        let diags = lint("parseInt('10', 10);");
        assert_eq!(diags.len(), 1, "should flag parseInt()");
    }

    #[test]
    fn test_allows_number_is_nan() {
        let diags = lint("Number.isNaN(x);");
        assert!(diags.is_empty(), "Number.isNaN() should not be flagged");
    }

    #[test]
    fn test_allows_number_parse_int() {
        let diags = lint("Number.parseInt('10', 10);");
        assert!(diags.is_empty(), "Number.parseInt() should not be flagged");
    }
}
