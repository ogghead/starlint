//! Rule: `prefer-event-target`
//!
//! Prefer `EventTarget` over Node.js `EventEmitter`. The `EventTarget` API
//! is a web standard available in browsers and modern Node.js, making code
//! more portable.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `new EventEmitter()` and `extends EventEmitter`.
#[derive(Debug)]
pub struct PreferEventTarget;

impl LintRule for PreferEventTarget {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-event-target".to_owned(),
            description: "Prefer `EventTarget` over `EventEmitter`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::Class, AstNodeType::NewExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::NewExpression(new_expr) => {
                let callee_info = ctx.node(new_expr.callee).and_then(|n| {
                    if let AstNode::IdentifierReference(id) = n {
                        if id.name.as_str() == "EventEmitter" {
                            return Some(Span::new(id.span.start, id.span.end));
                        }
                    }
                    None
                });
                if let Some(id_span) = callee_info {
                    ctx.report(Diagnostic {
                        rule_name: "prefer-event-target".to_owned(),
                        message: "Prefer `EventTarget` over `EventEmitter`".to_owned(),
                        span: Span::new(new_expr.span.start, new_expr.span.end),
                        severity: Severity::Warning,
                        help: Some("Replace `EventEmitter` with `EventTarget`".to_owned()),
                        fix: Some(Fix {
                            kind: FixKind::SuggestionFix,
                            message: "Replace `EventEmitter` with `EventTarget`".to_owned(),
                            edits: vec![Edit {
                                span: id_span,
                                replacement: "EventTarget".to_owned(),
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                }
            }
            AstNode::Class(class) => {
                let super_info = class
                    .super_class
                    .and_then(|sc_id| ctx.node(sc_id))
                    .and_then(|n| {
                        if let AstNode::IdentifierReference(id) = n {
                            if id.name.as_str() == "EventEmitter" {
                                return Some(Span::new(id.span.start, id.span.end));
                            }
                        }
                        None
                    });
                if let Some(id_span) = super_info {
                    ctx.report(Diagnostic {
                        rule_name: "prefer-event-target".to_owned(),
                        message: "Prefer extending `EventTarget` over `EventEmitter`".to_owned(),
                        span: Span::new(class.span.start, class.span.end),
                        severity: Severity::Warning,
                        help: Some("Replace `EventEmitter` with `EventTarget`".to_owned()),
                        fix: Some(Fix {
                            kind: FixKind::SuggestionFix,
                            message: "Replace `EventEmitter` with `EventTarget`".to_owned(),
                            edits: vec![Edit {
                                span: id_span,
                                replacement: "EventTarget".to_owned(),
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferEventTarget)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_new_event_emitter() {
        let diags = lint("var ee = new EventEmitter();");
        assert_eq!(diags.len(), 1, "new EventEmitter() should be flagged");
    }

    #[test]
    fn test_flags_extends_event_emitter() {
        let diags = lint("class Foo extends EventEmitter {}");
        assert_eq!(diags.len(), 1, "extends EventEmitter should be flagged");
    }

    #[test]
    fn test_allows_new_event_target() {
        let diags = lint("var et = new EventTarget();");
        assert!(diags.is_empty(), "new EventTarget() should not be flagged");
    }

    #[test]
    fn test_allows_extends_event_target() {
        let diags = lint("class Foo extends EventTarget {}");
        assert!(
            diags.is_empty(),
            "extends EventTarget should not be flagged"
        );
    }

    #[test]
    fn test_allows_plain_class() {
        let diags = lint("class Foo {}");
        assert!(diags.is_empty(), "plain class should not be flagged");
    }
}
