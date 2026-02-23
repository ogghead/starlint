//! Rule: `no-script-url`
//!
//! Disallow `javascript:` URLs. These are a form of `eval()` and pose
//! security risks (XSS).

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags string literals that contain `javascript:` URLs.
#[derive(Debug)]
pub struct NoScriptUrl;

impl LintRule for NoScriptUrl {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-script-url".to_owned(),
            description: "Disallow `javascript:` URLs".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::StringLiteral])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::StringLiteral(lit) = node else {
            return;
        };

        if lit.value.to_lowercase().starts_with("javascript:") {
            ctx.report(Diagnostic {
                rule_name: "no-script-url".to_owned(),
                message: "Script URL is a form of `eval()` and is a security risk".to_owned(),
                span: Span::new(lit.span.start, lit.span.end),
                severity: Severity::Error,
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
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoScriptUrl)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_javascript_url() {
        let diags = lint("var url = 'javascript:void(0)';");
        assert_eq!(diags.len(), 1, "javascript: URL should be flagged");
    }

    #[test]
    fn test_flags_javascript_url_mixed_case() {
        let diags = lint("var url = 'JavaScript:alert(1)';");
        assert_eq!(
            diags.len(),
            1,
            "mixed-case javascript: URL should be flagged"
        );
    }

    #[test]
    fn test_allows_normal_url() {
        let diags = lint("var url = 'https://example.com';");
        assert!(diags.is_empty(), "normal URL should not be flagged");
    }

    #[test]
    fn test_allows_non_url_string() {
        let diags = lint("var msg = 'hello world';");
        assert!(diags.is_empty(), "normal string should not be flagged");
    }
}
