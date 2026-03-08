//! Rule: `missing-throw` (OXC)
//!
//! Detect `new Error()` (or subclasses) used as an expression statement without
//! `throw`. Creating an error without throwing it is almost always a mistake.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Known error constructor names.
const ERROR_CONSTRUCTORS: &[&str] = &[
    "Error",
    "TypeError",
    "RangeError",
    "ReferenceError",
    "SyntaxError",
    "URIError",
    "EvalError",
    "AggregateError",
];

/// Flags `new Error()` without `throw`.
#[derive(Debug)]
pub struct MissingThrow;

impl LintRule for MissingThrow {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "missing-throw".to_owned(),
            description: "Detect `new Error()` without `throw`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ExpressionStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        // We look for ExpressionStatement where the expression is a NewExpression
        // with an error constructor callee.
        let AstNode::ExpressionStatement(stmt) = node else {
            return;
        };

        let Some(AstNode::NewExpression(new_expr)) = ctx.node(stmt.expression) else {
            return;
        };

        let is_error_ctor = match ctx.node(new_expr.callee) {
            Some(AstNode::IdentifierReference(id)) => {
                ERROR_CONSTRUCTORS.contains(&id.name.as_str())
            }
            _ => false,
        };

        if is_error_ctor {
            let stmt_span = Span::new(stmt.span.start, stmt.span.end);
            let new_expr_start = new_expr.span.start;
            ctx.report(Diagnostic {
                rule_name: "missing-throw".to_owned(),
                message: "`new Error()` is not thrown — did you forget `throw`?".to_owned(),
                span: stmt_span,
                severity: Severity::Warning,
                help: Some("Add `throw` before the expression".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Add `throw`".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(new_expr_start, new_expr_start),
                        replacement: "throw ".to_owned(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(MissingThrow)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_new_error_without_throw() {
        let diags = lint("new Error('oops');");
        assert_eq!(diags.len(), 1, "new Error without throw should be flagged");
    }

    #[test]
    fn test_flags_new_type_error_without_throw() {
        let diags = lint("new TypeError('bad');");
        assert_eq!(
            diags.len(),
            1,
            "new TypeError without throw should be flagged"
        );
    }

    #[test]
    fn test_allows_throw_new_error() {
        let diags = lint("throw new Error('oops');");
        assert!(diags.is_empty(), "throw new Error should not be flagged");
    }

    #[test]
    fn test_allows_assigned_error() {
        let diags = lint("const e = new Error('oops');");
        assert!(diags.is_empty(), "assigned new Error should not be flagged");
    }

    #[test]
    fn test_allows_non_error_constructor() {
        let diags = lint("new Map();");
        assert!(
            diags.is_empty(),
            "non-error constructor should not be flagged"
        );
    }
}
