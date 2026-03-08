//! Rule: `react/jsx-max-depth`
//!
//! Warn when JSX nesting exceeds a reasonable depth (default 10).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-max-depth";

/// Default maximum JSX nesting depth.
const DEFAULT_MAX_DEPTH: usize = 10;

/// Flags JSX elements that are nested deeper than the configured maximum depth.
/// Deep nesting is a code smell indicating the component should be broken up.
#[derive(Debug)]
pub struct JsxMaxDepth;

/// Recursively compute the maximum JSX nesting depth of an element's children.
/// Since children are `NodeId`s in `starlint_ast`, we need the `LintContext` to resolve them.
fn jsx_depth(children: &[NodeId], ctx: &LintContext<'_>) -> usize {
    let mut max = 0;
    for child_id in children {
        let child_depth = match ctx.node(*child_id) {
            Some(AstNode::JSXElement(el)) => jsx_depth(&el.children, ctx).saturating_add(1),
            Some(AstNode::JSXFragment(frag)) => jsx_depth(&frag.children, ctx),
            _ => 0,
        };
        if child_depth > max {
            max = child_depth;
        }
    }
    max
}

impl LintRule for JsxMaxDepth {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce a maximum JSX nesting depth".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXElement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXElement(element) = node else {
            return;
        };

        let depth = jsx_depth(&element.children, ctx).saturating_add(1);
        if depth > DEFAULT_MAX_DEPTH {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "JSX nesting depth of {depth} exceeds maximum of {DEFAULT_MAX_DEPTH}. Consider extracting sub-components"
                ),
                span: Span::new(element.span.start, element.span.end),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(JsxMaxDepth)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_allows_shallow_nesting() {
        let diags = lint("const el = <div><span><a>hi</a></span></div>;");
        assert!(
            diags.is_empty(),
            "should not flag shallow nesting (depth 3)"
        );
    }

    #[test]
    fn test_flags_deep_nesting() {
        // Build nesting of depth 11
        let mut source = String::from("const el = ");
        for _ in 0..11 {
            source.push_str("<div>");
        }
        source.push_str("hi");
        for _ in 0..11 {
            source.push_str("</div>");
        }
        source.push(';');
        let diags = lint(&source);
        assert!(
            !diags.is_empty(),
            "should flag nesting exceeding max depth of 10"
        );
    }

    #[test]
    fn test_allows_exactly_max_depth() {
        // Build nesting of exactly depth 10
        let mut source = String::from("const el = ");
        for _ in 0..10 {
            source.push_str("<div>");
        }
        source.push_str("hi");
        for _ in 0..10 {
            source.push_str("</div>");
        }
        source.push(';');
        let diags = lint(&source);
        assert!(
            diags.is_empty(),
            "should not flag nesting at exactly max depth"
        );
    }
}
