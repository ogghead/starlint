//! Rule: `error-message`
//!
//! Require error constructors to be called with a message argument.
//! `throw new Error()` without a message makes debugging harder.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Error constructors that should always have a message argument.
const ERROR_CONSTRUCTORS: &[&str] = &[
    "Error",
    "TypeError",
    "RangeError",
    "ReferenceError",
    "SyntaxError",
    "URIError",
    "EvalError",
];

/// Flags `new Error()` (and variants) without a message argument.
#[derive(Debug)]
pub struct ErrorMessage;

impl LintRule for ErrorMessage {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "error-message".to_owned(),
            description: "Require error constructors to have a message argument".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::NewExpression])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::NewExpression(new_expr) = node else {
            return;
        };

        let Some(AstNode::IdentifierReference(id)) = ctx.node(new_expr.callee) else {
            return;
        };

        let name = id.name.as_str();
        if !ERROR_CONSTRUCTORS.contains(&name) {
            return;
        }

        if !new_expr.arguments.is_empty() {
            return;
        }

        // Fix: insert a placeholder message string inside the parens
        // `new Error()` → `new Error('')`
        let fix = ctx
            .source_text()
            .get(new_expr.span.start as usize..new_expr.span.end as usize)
            .and_then(|text| {
                text.rfind(')').map(|paren_pos| {
                    let insert_pos = new_expr
                        .span
                        .start
                        .saturating_add(u32::try_from(paren_pos).unwrap_or(0));
                    Fix {
                        kind: FixKind::SuggestionFix,
                        message: "Add empty message `''`".to_owned(),
                        edits: vec![Edit {
                            span: Span::new(insert_pos, insert_pos),
                            replacement: "''".to_owned(),
                        }],
                        is_snippet: false,
                    }
                })
            });

        ctx.report(Diagnostic {
            rule_name: "error-message".to_owned(),
            message: format!("`new {name}()` should have a message argument"),
            span: Span::new(new_expr.span.start, new_expr.span.end),
            severity: Severity::Warning,
            help: None,
            fix,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    starlint_rule_framework::lint_rule_test!(ErrorMessage);

    #[test]
    fn test_flags_error_no_message() {
        let diags = lint("throw new Error();");
        assert_eq!(diags.len(), 1, "should flag new Error() without message");
    }

    #[test]
    fn test_flags_type_error_no_message() {
        let diags = lint("throw new TypeError();");
        assert_eq!(
            diags.len(),
            1,
            "should flag new TypeError() without message"
        );
    }

    #[test]
    fn test_allows_error_with_message() {
        let diags = lint("throw new Error('something went wrong');");
        assert!(diags.is_empty(), "Error with message should not be flagged");
    }

    #[test]
    fn test_allows_non_error_constructor() {
        let diags = lint("new MyClass();");
        assert!(
            diags.is_empty(),
            "non-error constructor should not be flagged"
        );
    }
}
