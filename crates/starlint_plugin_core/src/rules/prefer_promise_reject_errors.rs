//! Rule: `prefer-promise-reject-errors`
//!
//! Require using Error objects as Promise rejection reasons.
//! `Promise.reject('error')` should be `Promise.reject(new Error('error'))`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `Promise.reject()` calls with non-Error arguments.
#[derive(Debug)]
pub struct PreferPromiseRejectErrors;

impl LintRule for PreferPromiseRejectErrors {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-promise-reject-errors".to_owned(),
            description: "Require using Error objects as Promise rejection reasons".to_owned(),
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

        // Check for Promise.reject(...)
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property.as_str() != "reject" {
            return;
        }

        if !matches!(ctx.node(member.object), Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "Promise")
        {
            return;
        }

        // Check the first argument — flag if it's a literal (not an Error)
        if let Some(first_arg_id) = call.arguments.first() {
            let is_literal_rejection = matches!(
                ctx.node(*first_arg_id),
                Some(
                    AstNode::StringLiteral(_)
                        | AstNode::NumericLiteral(_)
                        | AstNode::BooleanLiteral(_)
                        | AstNode::NullLiteral(_)
                )
            );

            if is_literal_rejection {
                #[allow(clippy::as_conversions)]
                let fix = {
                    let arg_span = ctx.node(*first_arg_id).map_or(
                        starlint_ast::types::Span::EMPTY,
                        starlint_ast::AstNode::span,
                    );
                    let source = ctx.source_text();
                    source
                        .get(arg_span.start as usize..arg_span.end as usize)
                        .map(|arg_text| {
                            let replacement = format!("new Error({arg_text})");
                            Fix {
                                kind: FixKind::SuggestionFix,
                                message: format!("Replace with `{replacement}`"),
                                edits: vec![Edit {
                                    span: Span::new(arg_span.start, arg_span.end),
                                    replacement,
                                }],
                                is_snippet: false,
                            }
                        })
                };

                ctx.report(Diagnostic {
                    rule_name: "prefer-promise-reject-errors".to_owned(),
                    message: "Expected the Promise rejection reason to be an Error".to_owned(),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Warning,
                    help: Some(
                        "Wrap the rejection reason in `new Error(...)` for better stack traces"
                            .to_owned(),
                    ),
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
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferPromiseRejectErrors)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_string_rejection() {
        let diags = lint("Promise.reject('error');");
        assert_eq!(diags.len(), 1, "string rejection should be flagged");
    }

    #[test]
    fn test_allows_error_rejection() {
        let diags = lint("Promise.reject(new Error('error'));");
        assert!(diags.is_empty(), "Error rejection should not be flagged");
    }

    #[test]
    fn test_allows_variable_rejection() {
        let diags = lint("Promise.reject(err);");
        assert!(diags.is_empty(), "variable rejection should not be flagged");
    }
}
