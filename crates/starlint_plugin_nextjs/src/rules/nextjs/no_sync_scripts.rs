//! Rule: `nextjs/no-sync-scripts`
//!
//! Forbid synchronous scripts. Scripts without `async` or `defer` block
//! page rendering and hurt performance.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-sync-scripts";

/// Flags `<script src="...">` elements without `async` or `defer`.
#[derive(Debug)]
pub struct NoSyncScripts;

impl LintRule for NoSyncScripts {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid synchronous scripts".to_owned(),
            category: Category::Performance,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXOpeningElement])
    }

    #[allow(
        clippy::arithmetic_side_effects,
        clippy::as_conversions,
        clippy::cast_possible_truncation
    )]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXOpeningElement(opening) = node else {
            return;
        };

        // Only check lowercase `<script>` (HTML element)
        if opening.name.as_str() != "script" {
            return;
        }

        // Check if it has a `src` attribute
        let has_src = opening.attributes.iter().any(|attr_id| {
            if let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) {
                return attr.name.as_str() == "src";
            }
            false
        });

        if !has_src {
            return;
        }

        // Check for `async` or `defer` attributes
        let has_async_or_defer = opening.attributes.iter().any(|attr_id| {
            if let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) {
                let name = attr.name.as_str();
                return name == "async" || name == "defer";
            }
            false
        });

        if !has_async_or_defer {
            // Insert `async` after the element name — use end of opening name area
            // Since opening.name is a String, we approximate insertion after tag name
            // by computing from the span start + "<" + name length
            let name_len = opening.name.len() as u32;
            let insert_pos = opening.span.start + 1 + name_len;

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Synchronous scripts block page rendering -- add `async` or `defer` to `<script>` elements".to_owned(),
                span: Span::new(opening.span.start, opening.span.end),
                severity: Severity::Warning,
                help: None,
                fix: Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: "Add `async` attribute".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(insert_pos, insert_pos),
                        replacement: " async".to_owned(),
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
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoSyncScripts)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_sync_script() {
        let diags = lint(r#"const el = <script src="/script.js" />;"#);
        assert_eq!(diags.len(), 1, "sync script should be flagged");
    }

    #[test]
    fn test_allows_async_script() {
        let diags = lint(r#"const el = <script src="/script.js" async />;"#);
        assert!(diags.is_empty(), "async script should pass");
    }

    #[test]
    fn test_allows_defer_script() {
        let diags = lint(r#"const el = <script src="/script.js" defer />;"#);
        assert!(diags.is_empty(), "defer script should pass");
    }
}
