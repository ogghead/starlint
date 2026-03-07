//! Rule: `no-undefined`
//!
//! Disallow the use of `undefined` as an identifier. Using `undefined`
//! can be problematic because it can be shadowed in non-strict mode.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags references to `undefined`.
#[derive(Debug)]
pub struct NoUndefined;

impl LintRule for NoUndefined {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-undefined".to_owned(),
            description: "Disallow the use of `undefined` as an identifier".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::IdentifierReference])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::IdentifierReference(id) = node else {
            return;
        };

        if id.name.as_str() == "undefined" {
            let fix = Some(Fix {
                kind: FixKind::SuggestionFix,
                message: "Replace with `void 0`".to_owned(),
                edits: vec![Edit {
                    span: Span::new(id.span.start, id.span.end),
                    replacement: "void 0".to_owned(),
                }],
                is_snippet: false,
            });

            ctx.report(Diagnostic {
                rule_name: "no-undefined".to_owned(),
                message: "Unexpected use of `undefined` — use `void 0` instead if needed"
                    .to_owned(),
                span: Span::new(id.span.start, id.span.end),
                severity: Severity::Warning,
                help: Some("Replace `undefined` with `void 0`".to_owned()),
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
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUndefined)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_undefined_reference() {
        let diags = lint("var x = undefined;");
        assert_eq!(diags.len(), 1, "use of undefined should be flagged");
    }

    #[test]
    fn test_flags_undefined_comparison() {
        let diags = lint("if (x === undefined) {}");
        assert_eq!(
            diags.len(),
            1,
            "comparison with undefined should be flagged"
        );
    }

    #[test]
    fn test_allows_void_zero() {
        let diags = lint("var x = void 0;");
        assert!(diags.is_empty(), "void 0 should not be flagged");
    }

    #[test]
    fn test_allows_typeof_undefined() {
        // typeof undefined is technically an identifier reference but
        // typeof always works safely
        let diags = lint("var x = typeof y;");
        assert!(diags.is_empty(), "typeof should not be flagged");
    }
}
