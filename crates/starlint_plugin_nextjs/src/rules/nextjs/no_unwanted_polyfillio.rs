//! Rule: `nextjs/no-unwanted-polyfillio`
//!
//! Forbid polyfill.io scripts. The polyfill.io domain has been compromised
//! and should not be used. Next.js already includes necessary polyfills.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-unwanted-polyfillio";

/// Flags `<script>` elements that load from polyfill.io.
#[derive(Debug)]
pub struct NoUnwantedPolyfillio;

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

impl LintRule for NoUnwantedPolyfillio {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid polyfill.io scripts".to_owned(),
            category: Category::Performance,
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

        if opening.name.as_str() != "script" {
            return;
        }

        let has_polyfill_src = opening.attributes.iter().any(|attr_id| {
            if let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) {
                if attr.name.as_str() == "src" {
                    if let Some(val) = get_attr_string_value(attr, ctx) {
                        return val.contains("polyfill.io") || val.contains("polyfill.min.js");
                    }
                }
            }
            false
        });

        if has_polyfill_src {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Do not use polyfill.io -- it has been compromised. Next.js already includes necessary polyfills".to_owned(),
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

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUnwantedPolyfillio)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_polyfill_io() {
        let diags =
            lint(r#"const el = <script src="https://cdn.polyfill.io/v3/polyfill.min.js" />;"#);
        assert_eq!(diags.len(), 1, "polyfill.io script should be flagged");
    }

    #[test]
    fn test_allows_other_scripts() {
        let diags = lint(r#"const el = <script src="/my-script.js" />;"#);
        assert!(diags.is_empty(), "other scripts should not be flagged");
    }

    #[test]
    fn test_allows_script_component() {
        let diags = lint(r#"const el = <Script src="/my-script.js" />;"#);
        assert!(diags.is_empty(), "Script component should not be flagged");
    }
}
