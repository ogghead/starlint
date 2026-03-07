//! Rule: `prefer-numeric-literals`
//!
//! Disallow `parseInt()` and `Number.parseInt()` for binary, octal, and hex
//! literals. Use `0b`, `0o`, and `0x` prefix notation instead.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `parseInt(str, radix)` where radix is 2, 8, or 16.
#[derive(Debug)]
pub struct PreferNumericLiterals;

impl LintRule for PreferNumericLiterals {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-numeric-literals".to_owned(),
            description: "Disallow `parseInt()` for binary, octal, and hex literals".to_owned(),
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

        let callee_node = ctx.node(call.callee);
        let is_parse_int = match callee_node {
            Some(AstNode::IdentifierReference(id)) => id.name.as_str() == "parseInt",
            Some(AstNode::StaticMemberExpression(member)) => {
                member.property.as_str() == "parseInt"
                    && matches!(ctx.node(member.object), Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "Number")
            }
            _ => false,
        };

        if !is_parse_int || call.arguments.len() < 2 {
            return;
        }

        // Check if the second argument is a literal 2, 8, or 16
        let second_arg_id = call.arguments.get(1).copied();
        let second_arg = second_arg_id.and_then(|id| ctx.node(id));
        if let Some(AstNode::NumericLiteral(num)) = second_arg {
            let radix = num.value;
            let prefix = if (radix - 2.0).abs() < f64::EPSILON {
                Some("0b")
            } else if (radix - 8.0).abs() < f64::EPSILON {
                Some("0o")
            } else if (radix - 16.0).abs() < f64::EPSILON {
                Some("0x")
            } else {
                None
            };

            if let Some(lit_prefix) = prefix {
                // Extract string value from first argument
                let fix = call.arguments.first().and_then(|&arg_id| {
                    if let Some(AstNode::StringLiteral(s)) = ctx.node(arg_id) {
                        Some(Fix {
                            kind: FixKind::SafeFix,
                            message: "Use numeric literal".to_owned(),
                            edits: vec![Edit {
                                span: Span::new(call.span.start, call.span.end),
                                replacement: format!("{lit_prefix}{}", s.value.as_str()),
                            }],
                            is_snippet: false,
                        })
                    } else {
                        None
                    }
                });

                ctx.report(Diagnostic {
                    rule_name: "prefer-numeric-literals".to_owned(),
                    message: "Use a numeric literal instead of `parseInt()`".to_owned(),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Warning,
                    help: Some(format!("Use `{lit_prefix}` literal notation")),
                    fix,
                    labels: vec![],
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferNumericLiterals)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_hex_parse_int() {
        let diags = lint("parseInt('1A', 16);");
        assert_eq!(diags.len(), 1, "parseInt with radix 16 should be flagged");
    }

    #[test]
    fn test_flags_binary_parse_int() {
        let diags = lint("parseInt('111110111', 2);");
        assert_eq!(diags.len(), 1, "parseInt with radix 2 should be flagged");
    }

    #[test]
    fn test_flags_octal_parse_int() {
        let diags = lint("parseInt('767', 8);");
        assert_eq!(diags.len(), 1, "parseInt with radix 8 should be flagged");
    }

    #[test]
    fn test_allows_decimal_parse_int() {
        let diags = lint("parseInt('10', 10);");
        assert!(
            diags.is_empty(),
            "parseInt with radix 10 should not be flagged"
        );
    }
}
