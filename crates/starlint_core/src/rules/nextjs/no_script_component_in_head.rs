//! Rule: `nextjs/no-script-component-in-head`
//!
//! Forbid `<Script>` component inside `<Head>`. The `next/script` `<Script>`
//! component should not be placed within `next/head` `<Head>`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-script-component-in-head";

/// Flags `<Script>` components nested inside `<Head>`.
#[derive(Debug)]
pub struct NoScriptComponentInHead;

impl LintRule for NoScriptComponentInHead {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid `<Script>` inside `<Head>`".to_owned(),
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

        // Check if this is a <Head> element
        // element.opening_element is a NodeId
        let Some(AstNode::JSXOpeningElement(opening)) = ctx.node(element.opening_element) else {
            return;
        };
        if opening.name.as_str() != "Head" {
            return;
        }

        // Check children for <Script> components
        for child_id in &*element.children {
            if let Some(AstNode::JSXElement(child_element)) = ctx.node(*child_id) {
                if let Some(AstNode::JSXOpeningElement(child_opening)) =
                    ctx.node(child_element.opening_element)
                {
                    if child_opening.name.as_str() == "Script" {
                        ctx.report(Diagnostic {
                            rule_name: RULE_NAME.to_owned(),
                            message: "Do not use `<Script>` inside `<Head>` -- move `<Script>` outside of `<Head>`".to_owned(),
                            span: Span::new(
                                child_opening.span.start,
                                child_opening.span.end,
                            ),
                            severity: Severity::Error,
                            help: None,
                            fix: Some(Fix {
                                kind: FixKind::SuggestionFix,
                                message: "Remove `<Script>` from `<Head>`".to_owned(),
                                edits: vec![Edit {
                                    span: Span::new(
                                        child_element.span.start,
                                        child_element.span.end,
                                    ),
                                    replacement: String::new(),
                                }],
                                is_snippet: false,
                            }),
                            labels: vec![],
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoScriptComponentInHead)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_script_in_head() {
        let source = r#"const el = <Head><Script src="/script.js" /></Head>;"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "Script in Head should be flagged");
    }

    #[test]
    fn test_allows_script_outside_head() {
        let source =
            r#"const el = <><Head><title>Hi</title></Head><Script src="/script.js" /></>;"#;
        let diags = lint(source);
        assert!(diags.is_empty(), "Script outside Head should pass");
    }

    #[test]
    fn test_allows_lowercase_script_in_head() {
        let source = r#"const el = <Head><script src="/script.js" /></Head>;"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "lowercase script in Head should not be flagged by this rule"
        );
    }
}
