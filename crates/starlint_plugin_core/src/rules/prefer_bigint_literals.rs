//! Rule: `prefer-bigint-literals`
//!
//! Prefer `BigInt` literals (`123n`) over `BigInt(123)` constructor calls
//! for literal arguments. The literal syntax is shorter and clearer.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `BigInt(literal)` calls — prefer `BigInt` literal syntax instead.
#[derive(Debug)]
pub struct PreferBigintLiterals;

impl LintRule for PreferBigintLiterals {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-bigint-literals".to_owned(),
            description: "Prefer `BigInt` literals over `BigInt()` constructor calls".to_owned(),
            category: Category::Suggestion,
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

        // Must be a call to `BigInt`
        let Some(AstNode::IdentifierReference(id)) = ctx.node(call.callee) else {
            return;
        };

        if id.name.as_str() != "BigInt" {
            return;
        }

        // Must have exactly one argument
        let Some(&first_arg_id) = call.arguments.first() else {
            return;
        };

        if call.arguments.len() != 1 {
            return;
        }

        if let Some(literal_value) = get_bigint_literal_value(first_arg_id, ctx) {
            ctx.report(Diagnostic {
                rule_name: "prefer-bigint-literals".to_owned(),
                message:
                    "Prefer `BigInt` literal syntax (e.g. `123n`) over `BigInt()` with a literal argument"
                        .to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some(format!("Replace with `{literal_value}n`")),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: format!("Replace with `{literal_value}n`"),
                    edits: vec![Edit {
                        span: Span::new(call.span.start, call.span.end),
                        replacement: format!("{literal_value}n"),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Extract the literal value from a `BigInt()` argument, if it is a numeric or
/// pure-digit string literal suitable for `BigInt` literal syntax.
fn get_bigint_literal_value(arg_id: NodeId, ctx: &LintContext<'_>) -> Option<String> {
    match ctx.node(arg_id)? {
        AstNode::NumericLiteral(num) => {
            // Use raw source text for numeric literal
            let raw = &num.raw;
            if raw.is_empty() {
                None
            } else {
                Some(raw.clone())
            }
        }
        AstNode::StringLiteral(lit) => {
            let val = lit.value.as_str();
            (!val.is_empty() && val.chars().all(|c| c.is_ascii_digit())).then(|| val.to_owned())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferBigintLiterals);

    #[test]
    fn test_flags_bigint_with_numeric_literal() {
        let diags = lint("var x = BigInt(123);");
        assert_eq!(diags.len(), 1, "BigInt(123) should be flagged");
    }

    #[test]
    fn test_flags_bigint_with_string_digits() {
        let diags = lint("var x = BigInt(\"456\");");
        assert_eq!(diags.len(), 1, "BigInt with digit string should be flagged");
    }

    #[test]
    fn test_allows_bigint_literal() {
        let diags = lint("var x = 123n;");
        assert!(diags.is_empty(), "BigInt literal should not be flagged");
    }

    #[test]
    fn test_allows_bigint_with_variable() {
        let diags = lint("var x = BigInt(y);");
        assert!(
            diags.is_empty(),
            "BigInt with variable argument should not be flagged"
        );
    }

    #[test]
    fn test_allows_bigint_with_non_digit_string() {
        let diags = lint("var x = BigInt(\"0xff\");");
        assert!(
            diags.is_empty(),
            "BigInt with non-digit string should not be flagged"
        );
    }

    #[test]
    fn test_allows_bigint_no_args() {
        let diags = lint("var x = BigInt();");
        assert!(
            diags.is_empty(),
            "BigInt with no arguments should not be flagged"
        );
    }

    #[test]
    fn test_allows_other_function_call() {
        let diags = lint("var x = Number(123);");
        assert!(diags.is_empty(), "Number(123) should not be flagged");
    }
}
