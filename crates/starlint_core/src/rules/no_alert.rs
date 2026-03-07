//! Rule: `no-alert`
//!
//! Disallow the use of `alert`, `confirm`, and `prompt`. These are
//! browser-native dialogs that are generally bad UX and should not
//! appear in production code.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_utils;
use crate::lint_rule::{LintContext, LintRule};

/// Flags `alert()`, `confirm()`, and `prompt()` calls.
#[derive(Debug)]
pub struct NoAlert;

/// Blocked global function names.
const BLOCKED_NAMES: &[&str] = &["alert", "confirm", "prompt"];

impl LintRule for NoAlert {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-alert".to_owned(),
            description: "Disallow the use of `alert`, `confirm`, and `prompt`".to_owned(),
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

        let is_blocked = match ctx.node(call.callee) {
            Some(AstNode::IdentifierReference(id)) => BLOCKED_NAMES.contains(&id.name.as_str()),
            Some(AstNode::StaticMemberExpression(member)) => {
                BLOCKED_NAMES.contains(&member.property.as_str())
                    && matches!(
                        ctx.node(member.object),
                        Some(AstNode::IdentifierReference(id))
                            if id.name == "window" || id.name == "globalThis"
                    )
            }
            _ => false,
        };

        if !is_blocked {
            return;
        }

        let call_span = Span::new(call.span.start, call.span.end);
        let edit = fix_utils::delete_statement(ctx.source_text(), call_span);
        let fix = Some(Fix {
            kind: FixKind::SuggestionFix,
            message: "Remove this call".to_owned(),
            edits: vec![edit],
            is_snippet: false,
        });
        ctx.report(Diagnostic {
            rule_name: "no-alert".to_owned(),
            message: "Unexpected `alert`, `confirm`, or `prompt`".to_owned(),
            span: call_span,
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
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoAlert)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_alert() {
        let diags = lint("alert('hello');");
        assert_eq!(diags.len(), 1, "alert() should be flagged");
    }

    #[test]
    fn test_flags_confirm() {
        let diags = lint("confirm('sure?');");
        assert_eq!(diags.len(), 1, "confirm() should be flagged");
    }

    #[test]
    fn test_flags_prompt() {
        let diags = lint("prompt('name?');");
        assert_eq!(diags.len(), 1, "prompt() should be flagged");
    }

    #[test]
    fn test_flags_window_alert() {
        let diags = lint("window.alert('hello');");
        assert_eq!(diags.len(), 1, "window.alert() should be flagged");
    }

    #[test]
    fn test_allows_normal_function() {
        let diags = lint("doSomething();");
        assert!(diags.is_empty(), "normal function should not be flagged");
    }
}
