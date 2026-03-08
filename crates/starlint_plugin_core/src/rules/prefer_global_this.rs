//! Rule: `prefer-global-this` (unicorn)
//!
//! Prefer `globalThis` over `window`, `self`, or `global` for accessing
//! the global object. `globalThis` works in all environments.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Global object references that should be replaced with `globalThis`.
const DEPRECATED_GLOBALS: &[&str] = &["window", "self", "global"];

/// Flags references to `window`, `self`, or `global`.
#[derive(Debug)]
pub struct PreferGlobalThis;

impl LintRule for PreferGlobalThis {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-global-this".to_owned(),
            description: "Prefer globalThis over window/self/global".to_owned(),
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

        let name = ident.name.as_str();
        if DEPRECATED_GLOBALS.contains(&name) {
            let ident_span = Span::new(ident.span.start, ident.span.end);
            ctx.report(Diagnostic {
                rule_name: "prefer-global-this".to_owned(),
                message: format!("Prefer `globalThis` over `{name}`"),
                span: ident_span,
                severity: Severity::Warning,
                help: Some(format!("Replace `{name}` with `globalThis`")),
                fix: Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: format!("Replace `{name}` with `globalThis`"),
                    edits: vec![Edit {
                        span: ident_span,
                        replacement: "globalThis".to_owned(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferGlobalThis)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_window() {
        let diags = lint("var x = window.location;");
        assert_eq!(diags.len(), 1, "window should be flagged");
    }

    #[test]
    fn test_flags_global() {
        let diags = lint("var x = global.process;");
        assert_eq!(diags.len(), 1, "global should be flagged");
    }

    #[test]
    fn test_allows_global_this() {
        let diags = lint("var x = globalThis.location;");
        assert!(diags.is_empty(), "globalThis should not be flagged");
    }

    #[test]
    fn test_allows_other_identifiers() {
        let diags = lint("var x = foo.bar;");
        assert!(diags.is_empty(), "other identifiers should not be flagged");
    }
}
