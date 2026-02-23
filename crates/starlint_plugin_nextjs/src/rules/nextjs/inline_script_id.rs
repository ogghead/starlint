//! Rule: `nextjs/inline-script-id`
//!
//! Require `id` attribute on inline `<Script>` components from `next/script`.
//! Next.js uses the `id` to deduplicate inline scripts.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/inline-script-id";

/// Flags inline `<Script>` components missing an `id` attribute.
#[derive(Debug)]
pub struct InlineScriptId;

/// Check if an attribute with the given name exists.
fn has_attr_named(attributes: &[NodeId], name: &str, ctx: &LintContext<'_>) -> bool {
    attributes.iter().any(|&attr_id| {
        if let Some(AstNode::JSXAttribute(attr)) = ctx.node(attr_id) {
            attr.name == name
        } else {
            false
        }
    })
}

impl LintRule for InlineScriptId {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require `id` attribute on inline `<Script>` components".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXElement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXElement(element) = node else {
            return;
        };

        let Some(AstNode::JSXOpeningElement(opening)) = ctx.node(element.opening_element) else {
            return;
        };

        // Only check `<Script>` (PascalCase -- the Next.js component)
        if opening.name != "Script" {
            return;
        }

        let opening_span = opening.span;
        let attrs: Vec<NodeId> = opening.attributes.to_vec();

        // Check if it has dangerouslySetInnerHTML
        let has_dangerous = has_attr_named(&attrs, "dangerouslySetInnerHTML", ctx);

        // Check if it has a `src` attribute (external script)
        let has_src = has_attr_named(&attrs, "src", ctx);

        // An inline script has children or dangerouslySetInnerHTML but no src
        let is_inline = (!element.children.is_empty() || has_dangerous) && !has_src;

        if !is_inline {
            return;
        }

        // Require `id` attribute on inline scripts
        let has_id = has_attr_named(&attrs, "id", ctx);

        if !has_id {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Inline `<Script>` components require an `id` attribute for deduplication"
                    .to_owned(),
                span: Span::new(opening_span.start, opening_span.end),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(InlineScriptId)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_inline_script_without_id() {
        let diags = lint(r#"const el = <Script>{`console.log("hi")`}</Script>;"#);
        assert_eq!(diags.len(), 1, "inline Script without id should be flagged");
    }

    #[test]
    fn test_allows_inline_script_with_id() {
        let diags = lint(r#"const el = <Script id="my-script">{`console.log("hi")`}</Script>;"#);
        assert!(diags.is_empty(), "inline Script with id should pass");
    }

    #[test]
    fn test_allows_external_script() {
        let diags = lint(r#"const el = <Script src="/script.js"></Script>;"#);
        assert!(diags.is_empty(), "external Script should not require id");
    }
}
