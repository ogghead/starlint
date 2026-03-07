//! Rule: `react/no-danger`
//!
//! Flag `dangerouslySetInnerHTML` prop usage. Using `dangerouslySetInnerHTML`
//! bypasses React's DOM sanitization and exposes the application to XSS
//! attacks if the HTML content is not properly sanitized.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags usage of the `dangerouslySetInnerHTML` prop.
#[derive(Debug)]
pub struct NoDanger;

impl LintRule for NoDanger {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-danger".to_owned(),
            description: "Disallow usage of `dangerouslySetInnerHTML`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXAttribute])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXAttribute(attr) = node else {
            return;
        };

        if attr.name == "dangerouslySetInnerHTML" {
            let attr_span = Span::new(attr.span.start, attr.span.end);
            let fix = FixBuilder::new(
                "Remove `dangerouslySetInnerHTML` prop",
                FixKind::SuggestionFix,
            )
            .edit(fix_utils::remove_jsx_attr(ctx.source_text(), attr_span))
            .build();
            ctx.report(Diagnostic {
                rule_name: "react/no-danger".to_owned(),
                message:
                    "Avoid using `dangerouslySetInnerHTML` -- it exposes your app to XSS attacks"
                        .to_owned(),
                span: attr_span,
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
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoDanger)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_dangerously_set_inner_html() {
        let source = r#"var x = <div dangerouslySetInnerHTML={{ __html: "<b>bold</b>" }} />;"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "dangerouslySetInnerHTML should be flagged");
    }

    #[test]
    fn test_allows_normal_props() {
        let source = r#"var x = <div className="foo" />;"#;
        let diags = lint(source);
        assert!(diags.is_empty(), "normal props should not be flagged");
    }

    #[test]
    fn test_flags_on_custom_component() {
        let source = r#"var x = <MyComponent dangerouslySetInnerHTML={{ __html: "hi" }} />;"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "dangerouslySetInnerHTML on custom component should be flagged"
        );
    }
}
