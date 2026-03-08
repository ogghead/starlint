//! Rule: `nextjs/no-styled-jsx-in-document`
//!
//! Forbid styled-jsx in `_document`. The `<style jsx>` component does not
//! work correctly in `_document` because it is rendered on the server only.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-styled-jsx-in-document";

/// Flags `<style jsx>` elements in `_document` files.
#[derive(Debug)]
pub struct NoStyledJsxInDocument;

impl LintRule for NoStyledJsxInDocument {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid styled-jsx in `_document`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXOpeningElement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        // Only check in _document files
        let file_stem = ctx
            .file_path()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if file_stem != "_document" {
            return;
        }

        let AstNode::JSXOpeningElement(opening) = node else {
            return;
        };

        // opening.name is a String
        if opening.name.as_str() != "style" {
            return;
        }

        // Check for `jsx` attribute (boolean attribute)
        let has_jsx_attr = opening.attributes.iter().any(|attr_id| {
            matches!(
                ctx.node(*attr_id),
                Some(AstNode::JSXAttribute(attr)) if attr.name.as_str() == "jsx"
            )
        });

        if has_jsx_attr {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Styled-jsx (`<style jsx>`) should not be used in `_document`".to_owned(),
                span: Span::new(opening.span.start, opening.span.end),
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
    use starlint_rule_framework::lint_source;

    fn lint_with_path(source: &str, path: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoStyledJsxInDocument)];
        lint_source(source, path, &rules)
    }

    #[test]
    fn test_flags_styled_jsx_in_document() {
        let diags = lint_with_path(
            r"const el = <style jsx>{`.red { color: red; }`}</style>;",
            "pages/_document.tsx",
        );
        assert_eq!(diags.len(), 1, "styled-jsx in _document should be flagged");
    }

    #[test]
    fn test_allows_styled_jsx_in_page() {
        let diags = lint_with_path(
            r"const el = <style jsx>{`.red { color: red; }`}</style>;",
            "pages/index.tsx",
        );
        assert!(diags.is_empty(), "styled-jsx in page should pass");
    }

    #[test]
    fn test_allows_regular_style_in_document() {
        let diags = lint_with_path(
            r"const el = <style>{`.red { color: red; }`}</style>;",
            "pages/_document.tsx",
        );
        assert!(diags.is_empty(), "regular style in _document should pass");
    }
}
