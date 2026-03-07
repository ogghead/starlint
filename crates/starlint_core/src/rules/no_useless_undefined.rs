//! Rule: `no-useless-undefined` (unicorn)
//!
//! Disallow useless `undefined`. Using `undefined` as a default value,
//! return value, or argument is usually unnecessary since JavaScript
//! provides it implicitly.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags useless uses of `undefined`.
#[derive(Debug)]
pub struct NoUselessUndefined;

impl LintRule for NoUselessUndefined {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-undefined".to_owned(),
            description: "Disallow useless undefined".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::ReturnStatement,
            AstNodeType::VariableDeclarator,
        ])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            // `let x = undefined;` -> `let x;`
            AstNode::VariableDeclarator(decl) => {
                if let Some(init_id) = &decl.init {
                    if is_undefined(*init_id, ctx) {
                        // Remove from end of binding id to end of init (` = undefined`)
                        let id_span_end =
                            ctx.node(decl.id).map(|n| n.span().end).unwrap_or_default();
                        let init_span_end =
                            ctx.node(*init_id).map(|n| n.span().end).unwrap_or_default();
                        let remove_span = Span::new(id_span_end, init_span_end);
                        ctx.report(Diagnostic {
                            rule_name: "no-useless-undefined".to_owned(),
                            message: "Do not use useless `undefined`".to_owned(),
                            span: Span::new(decl.span.start, decl.span.end),
                            severity: Severity::Warning,
                            help: Some("Remove `= undefined`".to_owned()),
                            fix: Some(Fix {
                                kind: FixKind::SafeFix,
                                message: "Remove `= undefined`".to_owned(),
                                edits: vec![Edit {
                                    span: remove_span,
                                    replacement: String::new(),
                                }],
                                is_snippet: false,
                            }),
                            labels: vec![],
                        });
                    }
                }
            }
            // `return undefined;` -> `return;`
            AstNode::ReturnStatement(ret) => {
                if let Some(arg_id) = &ret.argument {
                    if is_undefined(*arg_id, ctx) {
                        // Remove from after `return` keyword to end of argument (` undefined`)
                        // `return` is 6 chars, so the keyword ends at ret.span.start + 6
                        let return_keyword_end = ret.span.start.saturating_add(6);
                        let arg_span_end =
                            ctx.node(*arg_id).map(|n| n.span().end).unwrap_or_default();
                        let remove_span = Span::new(return_keyword_end, arg_span_end);
                        ctx.report(Diagnostic {
                            rule_name: "no-useless-undefined".to_owned(),
                            message: "Do not use useless `undefined`".to_owned(),
                            span: Span::new(ret.span.start, ret.span.end),
                            severity: Severity::Warning,
                            help: Some("Remove `undefined` from return".to_owned()),
                            fix: Some(Fix {
                                kind: FixKind::SafeFix,
                                message: "Remove `undefined` from return".to_owned(),
                                edits: vec![Edit {
                                    span: remove_span,
                                    replacement: String::new(),
                                }],
                                is_snippet: false,
                            }),
                            labels: vec![],
                        });
                    }
                }
            }
            // `void 0` is a different pattern (intentional), skip it
            _ => {}
        }
    }
}

/// Check if a node is `undefined` (the identifier).
fn is_undefined(node_id: NodeId, ctx: &LintContext<'_>) -> bool {
    matches!(ctx.node(node_id), Some(AstNode::IdentifierReference(ident)) if ident.name == "undefined")
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUselessUndefined)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_let_undefined() {
        let diags = lint("let x = undefined;");
        assert_eq!(diags.len(), 1, "let x = undefined should be flagged");
    }

    #[test]
    fn test_flags_return_undefined() {
        let diags = lint("function foo() { return undefined; }");
        assert_eq!(diags.len(), 1, "return undefined should be flagged");
    }

    #[test]
    fn test_allows_let_with_value() {
        let diags = lint("let x = 1;");
        assert!(diags.is_empty(), "let with value should not be flagged");
    }

    #[test]
    fn test_allows_return_nothing() {
        let diags = lint("function foo() { return; }");
        assert!(diags.is_empty(), "bare return should not be flagged");
    }

    #[test]
    fn test_allows_let_no_init() {
        let diags = lint("let x;");
        assert!(diags.is_empty(), "let without init should not be flagged");
    }
}
