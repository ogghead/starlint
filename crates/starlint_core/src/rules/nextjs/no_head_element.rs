//! Rule: `nextjs/no-head-element`
//!
//! Forbid usage of the `<head>` HTML element. In Next.js, use the `<Head>`
//! component from `next/head` instead for proper SSR support.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-head-element";

/// Flags usage of the `<head>` HTML element.
#[derive(Debug)]
pub struct NoHeadElement;

impl LintRule for NoHeadElement {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid `<head>` HTML element, use `next/head` `<Head>` instead"
                .to_owned(),
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

        let is_head = opening.name.as_str() == "head";

        if is_head {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message:
                    "Do not use `<head>` -- use the `<Head>` component from `next/head` instead"
                        .to_owned(),
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
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoHeadElement)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_head_element() {
        let diags = lint(r"const el = <head><title>Hello</title></head>;");
        assert_eq!(diags.len(), 1, "<head> element should be flagged");
    }

    #[test]
    fn test_allows_head_component() {
        let diags = lint(r"const el = <Head><title>Hello</title></Head>;");
        assert!(diags.is_empty(), "<Head> component should not be flagged");
    }

    #[test]
    fn test_allows_other_elements() {
        let diags = lint(r"const el = <div>hello</div>;");
        assert!(diags.is_empty(), "other elements should not be flagged");
    }
}
