//! Rule: `jsx-a11y/iframe-has-title`
//!
//! Enforce `<iframe>` elements have a `title` attribute.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::fix_utils;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/iframe-has-title";

#[derive(Debug)]
pub struct IframeHasTitle;

impl LintRule for IframeHasTitle {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce `<iframe>` elements have a `title` attribute".to_owned(),
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

        if opening.name.as_str() != "iframe" {
            return;
        }

        let has_title = opening.attributes.iter().any(|&attr_id| {
            matches!(
                ctx.node(attr_id),
                Some(AstNode::JSXAttribute(attr)) if attr.name.as_str() == "title"
            )
        });

        if !has_title {
            let source = ctx.source_text();
            let end = usize::try_from(opening.span.end).unwrap_or(0);
            let insert_pos =
                if end > 1 && source.as_bytes().get(end.saturating_sub(2)) == Some(&b'/') {
                    opening.span.end.saturating_sub(2)
                } else {
                    opening.span.end.saturating_sub(1)
                };
            let fix = FixBuilder::new("Add `title` attribute", FixKind::SafeFix)
                .edit(fix_utils::insert_before(insert_pos, " title=\"\""))
                .build();
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "`<iframe>` elements must have a `title` attribute".to_owned(),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(IframeHasTitle)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_iframe_without_title() {
        let diags = lint(r#"const el = <iframe src="https://example.com" />;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_iframe_with_title() {
        let diags = lint(r#"const el = <iframe src="https://example.com" title="Example" />;"#);
        assert!(diags.is_empty());
    }
}
