//! Rule: `nextjs/next-script-for-ga`
//!
//! Suggest using `next/script` for Google Analytics instead of a raw
//! `<script>` element to benefit from Next.js script optimization.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/next-script-for-ga";

/// Known Google Analytics URL patterns.
const GA_PATTERNS: &[&str] = &[
    "www.google-analytics.com/analytics.js",
    "www.googletagmanager.com/gtag/js",
    "googletagmanager.com/gtm.js",
];

/// Flags raw `<script>` elements that load Google Analytics.
#[derive(Debug)]
pub struct NextScriptForGa;

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

impl LintRule for NextScriptForGa {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Use `next/script` for Google Analytics".to_owned(),
            category: Category::Suggestion,
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

        // Only check lowercase `<script>` (HTML element, not Next.js `<Script>`)
        if opening.name.as_str() != "script" {
            return;
        }

        // Check if src attribute contains a GA URL
        let has_ga_src = opening.attributes.iter().any(|attr_id| {
            if let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) {
                if attr.name.as_str() == "src" {
                    if let Some(val) = get_attr_string_value(attr, ctx) {
                        return GA_PATTERNS.iter().any(|pattern| val.contains(pattern));
                    }
                }
            }
            false
        });

        if has_ga_src {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Use the `<Script>` component from `next/script` for Google Analytics instead of a raw `<script>` element".to_owned(),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NextScriptForGa)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_ga_script() {
        let diags =
            lint(r#"const el = <script src="https://www.google-analytics.com/analytics.js" />;"#);
        assert_eq!(diags.len(), 1, "GA script should be flagged");
    }

    #[test]
    fn test_flags_gtag_script() {
        let diags = lint(
            r#"const el = <script src="https://www.googletagmanager.com/gtag/js?id=G-123" />;"#,
        );
        assert_eq!(diags.len(), 1, "gtag script should be flagged");
    }

    #[test]
    fn test_allows_non_ga_script() {
        let diags = lint(r#"const el = <script src="/my-script.js" />;"#);
        assert!(diags.is_empty(), "non-GA script should not be flagged");
    }
}
