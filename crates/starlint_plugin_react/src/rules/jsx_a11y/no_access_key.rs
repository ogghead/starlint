//! Rule: `jsx-a11y/no-access-key`
//!
//! Forbid `accessKey` attribute on elements.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::fix_utils;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/no-access-key";

#[derive(Debug)]
pub struct NoAccessKey;

impl LintRule for NoAccessKey {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid `accessKey` attribute on elements".to_owned(),
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

        let access_key_span = opening.attributes.iter().find_map(|attr_id| {
            if let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) {
                (attr.name.as_str() == "accessKey")
                    .then(|| Span::new(attr.span.start, attr.span.end))
            } else {
                None
            }
        });

        if let Some(attr_span) = access_key_span {
            let opening_span = Span::new(opening.span.start, opening.span.end);
            let source = ctx.source_text();
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Do not use the `accessKey` attribute. Access keys create inconsistent keyboard shortcuts across browsers".to_owned(),
                span: opening_span,
                severity: Severity::Warning,
                help: None,
                fix: FixBuilder::new("Remove `accessKey` attribute", FixKind::SuggestionFix)
                    .edit(fix_utils::remove_jsx_attr(source, attr_span))
                    .build(),
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoAccessKey);

    #[test]
    fn test_flags_access_key() {
        let diags = lint(r#"const el = <div accessKey="s">content</div>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_without_access_key() {
        let diags = lint(r"const el = <div>content</div>;");
        assert!(diags.is_empty());
    }
}
