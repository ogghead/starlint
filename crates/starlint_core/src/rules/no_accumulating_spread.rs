//! Rule: `no-accumulating-spread` (OXC)
//!
//! Detect spread operators used inside loops which create O(n^2) behavior.
//! For example, `result = [...result, item]` inside a loop copies the entire
//! array on each iteration. Use `push()` instead.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags spread in array expressions inside assignments (potential loop accumulation).
#[derive(Debug)]
pub struct NoAccumulatingSpread;

impl LintRule for NoAccumulatingSpread {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-accumulating-spread".to_owned(),
            description: "Detect spread operators that accumulate in loops (O(n^2))".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::AssignmentExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        // We look for assignments like `x = [...x, item]` or `x = {...x, key: val}`
        // These are O(n^2) when inside loops, but we flag them regardless as a warning
        // since they're almost always better written with push() or Object.assign().
        let AstNode::AssignmentExpression(assign) = node else {
            return;
        };

        let Some(AstNode::ArrayExpression(array)) = ctx.node(assign.right) else {
            return;
        };

        // Get the target name
        let target_name = match ctx.node(assign.left) {
            Some(AstNode::IdentifierReference(id)) => Some(id.name.clone()),
            _ => None,
        };

        let Some(target) = target_name else {
            return;
        };

        // Check if the array expression contains a spread of the same variable
        for &element_id in &*array.elements {
            let Some(AstNode::SpreadElement(spread)) = ctx.node(element_id) else {
                continue;
            };
            if let Some(AstNode::IdentifierReference(id)) = ctx.node(spread.argument) {
                if id.name.as_str() == target.as_str() {
                    ctx.report(Diagnostic {
                        rule_name: "no-accumulating-spread".to_owned(),
                        message: format!(
                            "`{target} = [...{target}, ...]` copies the entire array — \
                             use `{target}.push()` instead for better performance"
                        ),
                        span: Span::new(assign.span.start, assign.span.end),
                        severity: Severity::Warning,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoAccumulatingSpread)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_accumulating_spread() {
        let diags = lint("result = [...result, item];");
        assert_eq!(diags.len(), 1, "accumulating spread should be flagged");
    }

    #[test]
    fn test_flags_prepend_spread() {
        let diags = lint("result = [item, ...result];");
        assert_eq!(diags.len(), 1, "prepend spread should also be flagged");
    }

    #[test]
    fn test_allows_spread_of_different_variable() {
        let diags = lint("result = [...other, item];");
        assert!(
            diags.is_empty(),
            "spread of different variable should not be flagged"
        );
    }

    #[test]
    fn test_allows_push() {
        let diags = lint("result.push(item);");
        assert!(diags.is_empty(), "push should not be flagged");
    }
}
