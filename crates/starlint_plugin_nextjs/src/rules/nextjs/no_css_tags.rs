//! Rule: `nextjs/no-css-tags`
//!
//! Forbid `<link rel="stylesheet">` tags. In Next.js, CSS should be imported
//! via `import` statements so it can be optimized and code-split.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-css-tags";

/// Flags `<link rel="stylesheet">` elements.
#[derive(Debug)]
pub struct NoCssTags;

/// Get string value from a JSX attribute's value node.
fn get_attr_string_value(
    attr: &starlint_ast::node::JSXAttributeNode,
    ctx: &LintContext<'_>,
) -> Option<String> {
    let value_id = attr.value?;
    if let Some(AstNode::StringLiteral(lit)) = ctx.node(value_id) {
        Some(lit.value.clone())
    } else {
        None
    }
}

impl LintRule for NoCssTags {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid `<link rel=\"stylesheet\">` tags, use CSS imports instead"
                .to_owned(),
            category: Category::Style,
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

        if opening.name.as_str() != "link" {
            return;
        }

        let has_stylesheet_rel = opening.attributes.iter().any(|attr_id| {
            if let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) {
                if attr.name.as_str() == "rel" {
                    return get_attr_string_value(attr, ctx).as_deref() == Some("stylesheet");
                }
            }
            false
        });

        if has_stylesheet_rel {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Do not use `<link rel=\"stylesheet\">` -- use CSS `import` statements instead for Next.js optimization".to_owned(),
                span: Span::new(opening.span.start, opening.span.end),
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
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoCssTags)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_stylesheet_link() {
        let diags = lint(r#"const el = <link rel="stylesheet" href="/style.css" />;"#);
        assert_eq!(diags.len(), 1, "stylesheet link should be flagged");
    }

    #[test]
    fn test_allows_preconnect_link() {
        let diags = lint(r#"const el = <link rel="preconnect" href="https://example.com" />;"#);
        assert!(diags.is_empty(), "preconnect link should not be flagged");
    }

    #[test]
    fn test_allows_icon_link() {
        let diags = lint(r#"const el = <link rel="icon" href="/favicon.ico" />;"#);
        assert!(diags.is_empty(), "icon link should not be flagged");
    }
}
