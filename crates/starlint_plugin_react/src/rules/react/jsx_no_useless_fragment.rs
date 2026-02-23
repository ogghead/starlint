//! Rule: `react/jsx-no-useless-fragment`
//!
//! Warn when `<>child</>` or `<React.Fragment>child</React.Fragment>` wraps
//! only a single child element.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-no-useless-fragment";

/// Flags fragments (`<>...</>`) that wrap a single child, which is unnecessary.
#[derive(Debug)]
pub struct JsxNoUselessFragment;

/// Count meaningful children (skip whitespace-only text nodes).
fn meaningful_children_count(children: &[NodeId], ctx: &LintContext<'_>) -> usize {
    children
        .iter()
        .filter(|child_id| {
            if let Some(AstNode::JSXText(text)) = ctx.node(**child_id) {
                // Skip whitespace-only text nodes
                !text.value.as_str().trim().is_empty()
            } else {
                true
            }
        })
        .count()
}

impl LintRule for JsxNoUselessFragment {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow unnecessary fragments".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXFragment])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXFragment(fragment) = node else {
            return;
        };

        let count = meaningful_children_count(&fragment.children, ctx);
        if count <= 1 {
            let fragment_span = Span::new(fragment.span.start, fragment.span.end);

            // Build a fix for single-child case by extracting the child's source text
            let fix = if count == 1 {
                let source = ctx.source_text();
                fragment
                    .children
                    .iter()
                    .find(|child_id| {
                        if let Some(AstNode::JSXText(text)) = ctx.node(**child_id) {
                            !text.value.as_str().trim().is_empty()
                        } else {
                            true
                        }
                    })
                    .and_then(|child_id| {
                        let child_span = ctx.node(*child_id).map_or(
                            starlint_ast::types::Span::EMPTY,
                            starlint_ast::AstNode::span,
                        );
                        let start = usize::try_from(child_span.start).unwrap_or(0);
                        let end = usize::try_from(child_span.end).unwrap_or(0);
                        let child_text = source.get(start..end)?;
                        Some(Fix {
                            kind: FixKind::SafeFix,
                            message: "Remove the enclosing fragment".to_owned(),
                            edits: vec![Edit {
                                span: fragment_span,
                                replacement: child_text.to_owned(),
                            }],
                            is_snippet: false,
                        })
                    })
            } else {
                None
            };

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Unnecessary fragment: fragments with a single child (or no children) can be removed".to_owned(),
                span: fragment_span,
                severity: Severity::Warning,
                help: Some("Remove the unnecessary fragment wrapper".to_owned()),
                fix,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(JsxNoUselessFragment)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_single_child_fragment() {
        let diags = lint("const el = <><div /></>;");
        assert_eq!(diags.len(), 1, "should flag fragment with single child");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_flags_empty_fragment() {
        let diags = lint("const el = <></>;");
        assert_eq!(diags.len(), 1, "should flag empty fragment");
    }

    #[test]
    fn test_allows_multiple_children() {
        let diags = lint("const el = <><div /><span /></>;");
        assert!(
            diags.is_empty(),
            "should not flag fragment with multiple children"
        );
    }
}
