//! Rule: `jest/no-interpolation-in-snapshots`
//!
//! Error when template literals with expressions are used in inline snapshot arguments.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "jest/no-interpolation-in-snapshots";

/// Snapshot matcher names that should not receive interpolated template literals.
const SNAPSHOT_MATCHERS: &[&str] = &[
    "toMatchInlineSnapshot",
    "toThrowErrorMatchingInlineSnapshot",
];

/// Flags template literals with expressions used as inline snapshot arguments.
#[derive(Debug)]
pub struct NoInterpolationInSnapshots;

impl LintRule for NoInterpolationInSnapshots {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow template literal interpolation in inline snapshots".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Check for `.toMatchInlineSnapshot(...)` or `.toThrowErrorMatchingInlineSnapshot(...)`
        let is_snapshot_matcher = match ctx.node(call.callee) {
            Some(AstNode::StaticMemberExpression(member)) => {
                SNAPSHOT_MATCHERS.contains(&member.property.as_str())
            }
            _ => false,
        };

        if !is_snapshot_matcher {
            return;
        }

        // Check if any argument is a template literal with expressions
        for arg_id in &*call.arguments {
            if let Some(AstNode::TemplateLiteral(tmpl)) = ctx.node(*arg_id) {
                if !tmpl.expressions.is_empty() {
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: "Do not use template literal interpolation in inline snapshots — snapshots should be static strings".to_owned(),
                        span: Span::new(call.span.start, call.span.end),
                        severity: Severity::Error,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                    return;
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoInterpolationInSnapshots)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_interpolated_snapshot() {
        let source = "expect(value).toMatchInlineSnapshot(`value is ${x}`);";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "template literal with interpolation in snapshot should be flagged"
        );
    }

    #[test]
    fn test_flags_interpolated_throw_snapshot() {
        let source = "expect(fn).toThrowErrorMatchingInlineSnapshot(`error: ${msg}`);";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "interpolation in toThrowErrorMatchingInlineSnapshot should be flagged"
        );
    }

    #[test]
    fn test_allows_static_template_literal() {
        let source = "expect(value).toMatchInlineSnapshot(`static value`);";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "static template literal without interpolation should not be flagged"
        );
    }
}
