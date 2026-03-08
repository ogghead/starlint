//! Rule: `react/iframe-missing-sandbox`
//!
//! Warn when `<iframe>` elements don't have a `sandbox` attribute.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::fix_utils;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `<iframe>` JSX elements that lack a `sandbox` attribute.
/// The `sandbox` attribute restricts iframe capabilities and is an
/// important security measure.
#[derive(Debug)]
pub struct IframeMissingSandbox;

impl LintRule for IframeMissingSandbox {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/iframe-missing-sandbox".to_owned(),
            description: "Require sandbox attribute on iframe elements".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXOpeningElement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXOpeningElement(opening) = node else {
            return;
        };

        // Check if element is an `iframe`
        if opening.name.as_str() != "iframe" {
            return;
        }

        // Check for `sandbox` attribute
        let has_sandbox = opening
            .attributes
            .iter()
            .any(|&attr_id| match ctx.node(attr_id) {
                Some(AstNode::JSXAttribute(a)) => a.name.as_str() == "sandbox",
                _ => false,
            });

        if !has_sandbox {
            // Insert `sandbox=""` before the closing `>` or `/>`
            let source = ctx.source_text();
            let end = usize::try_from(opening.span.end).unwrap_or(0);
            let insert_pos =
                if end > 1 && source.as_bytes().get(end.saturating_sub(2)) == Some(&b'/') {
                    // Self-closing: insert before `/>`
                    opening.span.end.saturating_sub(2)
                } else {
                    // Regular: insert before `>`
                    opening.span.end.saturating_sub(1)
                };
            let fix = FixBuilder::new("Add `sandbox` attribute", FixKind::SuggestionFix)
                .edit(fix_utils::insert_before(insert_pos, " sandbox=\"\""))
                .build();
            ctx.report(Diagnostic {
                rule_name: "react/iframe-missing-sandbox".to_owned(),
                message: "`<iframe>` elements should have a `sandbox` attribute for security"
                    .to_owned(),
                span: Span::new(opening.span.start, opening.span.end),
                severity: Severity::Warning,
                help: None,
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(IframeMissingSandbox)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_iframe_without_sandbox() {
        let source = r#"const x = <iframe src="https://example.com" />;"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "iframe without sandbox should be flagged");
    }

    #[test]
    fn test_allows_iframe_with_sandbox() {
        let source = r#"const x = <iframe src="https://example.com" sandbox="allow-scripts" />;"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "iframe with sandbox should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_iframe_element() {
        let source = "const x = <div />;";
        let diags = lint(source);
        assert!(diags.is_empty(), "non-iframe element should not be flagged");
    }
}
