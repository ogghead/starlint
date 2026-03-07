//! Rule: `no-useless-constructor`
//!
//! Disallow unnecessary constructors. An empty constructor or one that simply
//! delegates to `super()` with the same arguments is unnecessary.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::MethodDefinitionKind;
use starlint_ast::types::NodeId;

/// Flags constructors that don't do anything useful.
#[derive(Debug)]
pub struct NoUselessConstructor;

impl LintRule for NoUselessConstructor {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-constructor".to_owned(),
            description: "Disallow unnecessary constructors".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::Class])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::Class(class) = node else {
            return;
        };

        let has_super = class.super_class.is_some();

        for element_id in &class.body {
            let Some(AstNode::MethodDefinition(method)) = ctx.node(*element_id) else {
                continue;
            };

            if method.kind != MethodDefinitionKind::Constructor {
                continue;
            }

            let method_span = method.span;
            let value_id = method.value;
            let Some(AstNode::Function(func)) = ctx.node(value_id) else {
                continue;
            };

            let Some(body_id) = func.body else {
                continue;
            };

            let params = func.params.clone();

            let Some(AstNode::BlockStatement(body)) = ctx.node(body_id) else {
                continue;
            };
            let body_stmts = body.body.clone();

            // Empty constructor with no super class
            if body_stmts.is_empty() && !has_super {
                let method_span_diag = Span::new(method_span.start, method_span.end);
                ctx.report(Diagnostic {
                    rule_name: "no-useless-constructor".to_owned(),
                    message: "Useless constructor — empty constructor is unnecessary".to_owned(),
                    span: method_span_diag,
                    severity: Severity::Error,
                    help: Some("Remove the empty constructor".to_owned()),
                    fix: Some(Fix {
                        kind: FixKind::SafeFix,
                        message: "Remove empty constructor".to_owned(),
                        edits: vec![Edit {
                            span: method_span_diag,
                            replacement: String::new(),
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
                continue;
            }

            // Constructor that only calls super(...args) with same params
            if has_super && body_stmts.len() == 1 {
                if let Some(AstNode::ExpressionStatement(expr_stmt)) =
                    body_stmts.first().and_then(|id| ctx.node(*id))
                {
                    if is_simple_super_call(expr_stmt.expression, &params, ctx) {
                        let method_span_diag = Span::new(method_span.start, method_span.end);
                        ctx.report(Diagnostic {
                            rule_name: "no-useless-constructor".to_owned(),
                            message: "Useless constructor — constructor simply delegates to `super()` with the same arguments".to_owned(),
                            span: method_span_diag,
                            severity: Severity::Error,
                            help: Some("Remove the useless constructor".to_owned()),
                            fix: Some(Fix {
                                kind: FixKind::SafeFix,
                                message: "Remove useless constructor".to_owned(),
                                edits: vec![Edit {
                                    span: method_span_diag,
                                    replacement: String::new(),
                                }],
                                is_snippet: false,
                            }),
                            labels: vec![],
                        });
                    }
                }
            }
        }
    }
}

/// Check if an expression is a `super(...)` call that passes through exactly
/// the same parameters.
fn is_simple_super_call(expr_id: NodeId, params: &[NodeId], ctx: &LintContext<'_>) -> bool {
    let Some(AstNode::CallExpression(call)) = ctx.node(expr_id) else {
        return false;
    };

    // Must be a super() call — check source text since AstNode::Super doesn't exist
    let callee_node = ctx.node(call.callee);
    let is_super = callee_node.is_some_and(|n| {
        let sp = n.span();
        let start = usize::try_from(sp.start).unwrap_or(0);
        let end = usize::try_from(sp.end).unwrap_or(0);
        ctx.source_text().get(start..end) == Some("super")
    });
    if !is_super {
        return false;
    }

    let param_count = params.len();

    // Check if super() is called with the exact same number of arguments
    if call.arguments.len() != param_count {
        return false;
    }

    // For zero params, super() is a simple passthrough
    if param_count == 0 && call.arguments.is_empty() {
        return true;
    }

    // Check if each argument is a simple identifier matching the param
    for (arg_id, param_id) in call.arguments.iter().zip(params.iter()) {
        let Some(AstNode::IdentifierReference(arg_ident)) = ctx.node(*arg_id) else {
            return false;
        };
        let Some(AstNode::BindingIdentifier(param_ident)) = ctx.node(*param_id) else {
            return false;
        };
        if arg_ident.name != param_ident.name {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUselessConstructor)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_empty_constructor() {
        let diags = lint("class A { constructor() {} }");
        assert_eq!(diags.len(), 1, "empty constructor should be flagged");
    }

    #[test]
    fn test_flags_super_only_constructor() {
        let diags = lint("class B extends A { constructor() { super(); } }");
        assert_eq!(
            diags.len(),
            1,
            "constructor that only calls super() should be flagged"
        );
    }

    #[test]
    fn test_flags_super_passthrough() {
        let diags = lint("class B extends A { constructor(x, y) { super(x, y); } }");
        assert_eq!(
            diags.len(),
            1,
            "constructor that passes through args to super() should be flagged"
        );
    }

    #[test]
    fn test_allows_constructor_with_body() {
        let diags = lint("class A { constructor() { this.x = 1; } }");
        assert!(
            diags.is_empty(),
            "constructor with body should not be flagged"
        );
    }

    #[test]
    fn test_allows_constructor_with_extra_logic() {
        let diags = lint("class B extends A { constructor() { super(); this.x = 1; } }");
        assert!(
            diags.is_empty(),
            "constructor with super + extra logic should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_constructor() {
        let diags = lint("class A { method() {} }");
        assert!(
            diags.is_empty(),
            "class without constructor should not be flagged"
        );
    }
}
