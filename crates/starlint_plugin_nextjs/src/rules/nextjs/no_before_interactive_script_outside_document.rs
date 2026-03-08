//! Rule: `nextjs/no-before-interactive-script-outside-document`
//!
//! Forbid `strategy="beforeInteractive"` on `<Script>` outside of `_document`.
//! The `beforeInteractive` strategy only works in `pages/_document`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-before-interactive-script-outside-document";

/// Flags `<Script strategy="beforeInteractive">` outside of `_document` files.
#[derive(Debug)]
pub struct NoBeforeInteractiveScriptOutsideDocument;

/// Get string value from a JSX attribute's value node.
fn get_string_value(ctx: &LintContext<'_>, value: Option<NodeId>) -> Option<String> {
    let id = value?;
    let node = ctx.node(id)?;
    if let AstNode::StringLiteral(lit) = node {
        Some(lit.value.clone())
    } else {
        None
    }
}

impl LintRule for NoBeforeInteractiveScriptOutsideDocument {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid `strategy=\"beforeInteractive\"` outside `_document`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXOpeningElement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXOpeningElement(opening) = node else {
            return;
        };

        // Only check `<Script>` (PascalCase component)
        if opening.name.as_str() != "Script" {
            return;
        }

        // Check for strategy="beforeInteractive"
        let has_before_interactive = opening.attributes.iter().any(|attr_id| {
            let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) else {
                return false;
            };
            if attr.name.as_str() == "strategy" {
                return get_string_value(ctx, attr.value).as_deref() == Some("beforeInteractive");
            }
            false
        });

        if !has_before_interactive {
            return;
        }

        // Check if the file is _document
        let file_path = ctx.file_path();
        let file_stem = file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

        if file_stem != "_document" {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "`strategy=\"beforeInteractive\"` on `<Script>` is only allowed in `pages/_document`".to_owned(),
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
    use std::path::Path;

    fn lint_with_path(source: &str, path: &Path) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> =
            vec![Box::new(NoBeforeInteractiveScriptOutsideDocument)];
        lint_source(source, path.to_str().unwrap_or("test.js"), &rules)
    }

    #[test]
    fn test_flags_before_interactive_outside_document() {
        let diags = lint_with_path(
            r#"const el = <Script strategy="beforeInteractive" src="/script.js" />;"#,
            Path::new("pages/index.tsx"),
        );
        assert_eq!(
            diags.len(),
            1,
            "beforeInteractive outside _document should be flagged"
        );
    }

    #[test]
    fn test_allows_before_interactive_in_document() {
        let diags = lint_with_path(
            r#"const el = <Script strategy="beforeInteractive" src="/script.js" />;"#,
            Path::new("pages/_document.tsx"),
        );
        assert!(
            diags.is_empty(),
            "beforeInteractive in _document should pass"
        );
    }

    #[test]
    fn test_allows_other_strategies() {
        let diags = lint_with_path(
            r#"const el = <Script strategy="afterInteractive" src="/script.js" />;"#,
            Path::new("pages/index.tsx"),
        );
        assert!(diags.is_empty(), "other strategies should not be flagged");
    }
}
