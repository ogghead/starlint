//! Rule: `no-null` (unicorn)
//!
//! Disallow the use of `null`. Prefer `undefined` for consistency.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags use of `null`.
#[derive(Debug)]
pub struct NoNull;

impl LintRule for NoNull {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-null".to_owned(),
            description: "Disallow the use of `null`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::NullLiteral])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::NullLiteral(lit) = node else {
            return;
        };

        ctx.report(Diagnostic {
            rule_name: "no-null".to_owned(),
            message: "Avoid using `null` — prefer `undefined` for consistency".to_owned(),
            span: Span::new(lit.span.start, lit.span.end),
            severity: Severity::Warning,
            help: Some("Replace `null` with `undefined`".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SuggestionFix,
                message: "Replace `null` with `undefined`".to_owned(),
                edits: vec![Edit {
                    span: Span::new(lit.span.start, lit.span.end),
                    replacement: "undefined".to_owned(),
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
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoNull)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_null() {
        let diags = lint("var x = null;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_undefined() {
        let diags = lint("var x = undefined;");
        assert!(diags.is_empty());
    }
}
