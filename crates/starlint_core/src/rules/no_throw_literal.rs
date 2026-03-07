//! Rule: `no-throw-literal`
//!
//! Restrict what can be thrown as an exception. Only `Error` objects (or
//! subclasses) should be thrown because they capture a stack trace.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags `throw` statements that throw non-Error values.
#[derive(Debug)]
pub struct NoThrowLiteral;

impl LintRule for NoThrowLiteral {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-throw-literal".to_owned(),
            description: "Disallow throwing literals and non-Error objects".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ThrowStatement])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ThrowStatement(throw) = node else {
            return;
        };

        let Some(arg_node) = ctx.node(throw.argument) else {
            return;
        };

        if !is_literal_or_non_error(arg_node) {
            return;
        }

        let arg_span = arg_node.span();
        let source = ctx.source_text();
        let fix = source
            .get(arg_span.start as usize..arg_span.end as usize)
            .map(|arg_text| {
                let replacement = format!("new Error({arg_text})");
                Fix {
                    kind: FixKind::SuggestionFix,
                    message: format!("Replace with `throw {replacement}`"),
                    edits: vec![Edit {
                        span: Span::new(arg_span.start, arg_span.end),
                        replacement,
                    }],
                    is_snippet: false,
                }
            });

        ctx.report(Diagnostic {
            rule_name: "no-throw-literal".to_owned(),
            message: "Expected an Error object to be thrown".to_owned(),
            span: Span::new(throw.span.start, throw.span.end),
            severity: Severity::Error,
            help: Some(
                "Wrap the thrown value in `new Error(...)` for better stack traces".to_owned(),
            ),
            fix,
            labels: vec![],
        });
    }
}

/// Check if a node is a literal value (string, number, boolean, null,
/// template literal, object, array) rather than an Error object.
const fn is_literal_or_non_error(node: &AstNode) -> bool {
    matches!(
        node,
        AstNode::StringLiteral(_)
            | AstNode::NumericLiteral(_)
            | AstNode::BooleanLiteral(_)
            | AstNode::NullLiteral(_)
            | AstNode::TemplateLiteral(_)
            | AstNode::ObjectExpression(_)
            | AstNode::ArrayExpression(_)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoThrowLiteral)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_throw_string() {
        let diags = lint("throw 'error';");
        assert_eq!(
            diags.len(),
            1,
            "throwing a string literal should be flagged"
        );
    }

    #[test]
    fn test_flags_throw_number() {
        let diags = lint("throw 0;");
        assert_eq!(diags.len(), 1, "throwing a number should be flagged");
    }

    #[test]
    fn test_flags_throw_null() {
        let diags = lint("throw null;");
        assert_eq!(diags.len(), 1, "throwing null should be flagged");
    }

    #[test]
    fn test_allows_throw_new_error() {
        let diags = lint("throw new Error('msg');");
        assert!(diags.is_empty(), "throwing new Error should not be flagged");
    }

    #[test]
    fn test_allows_throw_variable() {
        let diags = lint("throw err;");
        assert!(
            diags.is_empty(),
            "throwing a variable should not be flagged"
        );
    }

    #[test]
    fn test_allows_throw_call() {
        let diags = lint("throw getError();");
        assert!(
            diags.is_empty(),
            "throwing a function call should not be flagged"
        );
    }
}
