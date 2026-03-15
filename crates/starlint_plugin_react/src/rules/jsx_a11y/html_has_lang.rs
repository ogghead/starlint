//! Rule: `jsx-a11y/html-has-lang`
//!
//! Enforce `<html>` element has a `lang` attribute.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::fix_utils;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/html-has-lang";

#[derive(Debug)]
pub struct HtmlHasLang;

impl LintRule for HtmlHasLang {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce `<html>` element has a `lang` attribute".to_owned(),
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

        if opening.name.as_str() != "html" {
            return;
        }

        let has_lang = opening.attributes.iter().any(|&attr_id| {
            matches!(
                ctx.node(attr_id),
                Some(AstNode::JSXAttribute(attr)) if attr.name.as_str() == "lang"
            )
        });

        if !has_lang {
            let source = ctx.source_text();
            let end = usize::try_from(opening.span.end).unwrap_or(0);
            let insert_pos =
                if end > 1 && source.as_bytes().get(end.saturating_sub(2)) == Some(&b'/') {
                    opening.span.end.saturating_sub(2)
                } else {
                    opening.span.end.saturating_sub(1)
                };
            let fix = FixBuilder::new("Add `lang` attribute", FixKind::SafeFix)
                .edit(fix_utils::insert_before(insert_pos, " lang=\"en\""))
                .build();
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "`<html>` elements must have a `lang` attribute".to_owned(),
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

    starlint_rule_framework::lint_rule_test!(HtmlHasLang);

    #[test]
    fn test_flags_html_without_lang() {
        let diags = lint(r"const el = <html><body>content</body></html>;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_html_with_lang() {
        let diags = lint(r#"const el = <html lang="en"><body>content</body></html>;"#);
        assert!(diags.is_empty());
    }
}
