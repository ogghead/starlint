//! Rule: `default-param-last`
//!
//! Enforce default parameters to be last. Non-default parameters after
//! a default parameter cannot take advantage of defaults without passing
//! `undefined` explicitly.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags default parameters that are not in the last positions.
#[derive(Debug)]
pub struct DefaultParamLast;

impl LintRule for DefaultParamLast {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "default-param-last".to_owned(),
            description: "Enforce default parameters to be last".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ArrowFunctionExpression, AstNodeType::Function])
    }

    #[allow(clippy::indexing_slicing)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let params = match node {
            AstNode::Function(f) => &f.params,
            AstNode::ArrowFunctionExpression(arrow) => &arrow.params,
            _ => return,
        };

        if params.is_empty() {
            return;
        }

        // Check whether each param has a default by testing if it is an
        // AssignmentPattern node (produced by the converter for `x = value`).
        let has_default: Vec<bool> = params
            .iter()
            .map(|pid| matches!(ctx.node(*pid), Some(AstNode::AssignmentPattern(_))))
            .collect();

        // Find the last non-default, non-rest parameter.
        // Any default parameter before it is a violation.
        let mut seen_non_default = false;
        for (i, is_default) in has_default.iter().enumerate().rev() {
            if *is_default {
                if seen_non_default {
                    if let Some(param_node) = ctx.node(params[i]) {
                        let ps = param_node.span();
                        ctx.report(Diagnostic {
                            rule_name: "default-param-last".to_owned(),
                            message: "Default parameters should be last".to_owned(),
                            span: Span::new(ps.start, ps.end),
                            severity: Severity::Warning,
                            help: None,
                            fix: None,
                            labels: vec![],
                        });
                    }
                }
            } else {
                seen_non_default = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(DefaultParamLast);

    #[test]
    fn test_allows_defaults_at_end() {
        let diags = lint("function foo(a, b = 1) {}");
        assert!(diags.is_empty(), "default at end should not be flagged");
    }

    #[test]
    fn test_flags_default_before_non_default() {
        let diags = lint("function foo(a = 1, b) {}");
        assert_eq!(
            diags.len(),
            1,
            "default before non-default should be flagged"
        );
    }

    #[test]
    fn test_allows_all_defaults() {
        let diags = lint("function foo(a = 1, b = 2) {}");
        assert!(diags.is_empty(), "all defaults should not be flagged");
    }

    #[test]
    fn test_allows_no_defaults() {
        let diags = lint("function foo(a, b) {}");
        assert!(diags.is_empty(), "no defaults should not be flagged");
    }

    #[test]
    fn test_flags_arrow_function() {
        let diags = lint("const foo = (a = 1, b) => {};");
        assert_eq!(
            diags.len(),
            1,
            "arrow with default before non-default should be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_violations() {
        let diags = lint("function foo(a = 1, b = 2, c) {}");
        assert_eq!(
            diags.len(),
            2,
            "multiple defaults before non-default should all be flagged"
        );
    }
}
