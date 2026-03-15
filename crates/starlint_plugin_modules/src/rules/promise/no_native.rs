//! Rule: `promise/no-native`
//!
//! Forbid use of the native `Promise` global. Useful when a project
//! requires a polyfill (e.g. `bluebird`) for consistency or extra features.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags any reference to the `Promise` identifier.
#[derive(Debug)]
pub struct NoNative;

impl LintRule for NoNative {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/no-native".to_owned(),
            description: "Forbid native `Promise` (enforce polyfill)".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::IdentifierReference])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::IdentifierReference(ident) = node else {
            return;
        };

        if ident.name.as_str() == "Promise" {
            ctx.report(Diagnostic {
                rule_name: "promise/no-native".to_owned(),
                message: "Avoid using native `Promise` — use the configured polyfill instead"
                    .to_owned(),
                span: Span::new(ident.span.start, ident.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoNative);

    #[test]
    fn test_flags_promise_resolve() {
        let diags = lint("const p = Promise.resolve(1);");
        assert!(!diags.is_empty(), "should flag native Promise usage");
    }

    #[test]
    fn test_flags_new_promise() {
        let diags = lint("const p = new Promise((r) => r(1));");
        assert!(!diags.is_empty(), "should flag new Promise");
    }

    #[test]
    fn test_allows_non_promise() {
        let diags = lint("const m = new Map();");
        assert!(diags.is_empty(), "non-Promise should not be flagged");
    }
}
