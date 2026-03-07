//! Rule: `no-magic-array-flat-depth`
//!
//! Flag `.flat(n)` calls where `n` is a numeric literal greater than 1.
//! Non-trivial flat depths are magic numbers that should be extracted to
//! named constants. `.flat()`, `.flat(1)`, `.flat(Infinity)`, and
//! `.flat(depth)` (variable) are all acceptable.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `.flat(n)` calls with magic number depths greater than 1.
#[derive(Debug)]
pub struct NoMagicArrayFlatDepth;

impl LintRule for NoMagicArrayFlatDepth {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-magic-array-flat-depth".to_owned(),
            description: "Disallow magic number depths in `Array.prototype.flat()` calls"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Check for `.flat(...)` member call
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property.as_str() != "flat" {
            return;
        }

        // Must have exactly one argument
        if call.arguments.len() != 1 {
            return;
        }

        let Some(first_arg_id) = call.arguments.first() else {
            return;
        };

        // Only flag numeric literals > 1
        // Allow: .flat(), .flat(1), .flat(Infinity), .flat(someVar)
        if is_magic_flat_depth(*first_arg_id, ctx) {
            ctx.report(Diagnostic {
                rule_name: "no-magic-array-flat-depth".to_owned(),
                message: "Magic number depth in `.flat()` — use a named constant for non-trivial flat depths".to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if an argument is a numeric literal with value > 1.
///
/// Returns `false` for non-numeric arguments (variables, `Infinity`, etc.).
fn is_magic_flat_depth(arg_id: NodeId, ctx: &LintContext<'_>) -> bool {
    match ctx.node(arg_id) {
        Some(AstNode::NumericLiteral(num)) => {
            // Allow .flat(1) — this is the default depth
            // Flag .flat(2), .flat(3), .flat(5), etc.
            num.value > 1.0 && num.value.is_finite()
        }
        // .flat(Infinity) is a common pattern — allow it
        // .flat(someVariable) is fine — it's not a magic number
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoMagicArrayFlatDepth)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_flat_with_magic_number() {
        let diags = lint("arr.flat(3);");
        assert_eq!(diags.len(), 1, "flat(3) should be flagged as magic number");
    }

    #[test]
    fn test_flags_flat_with_depth_two() {
        let diags = lint("arr.flat(2);");
        assert_eq!(diags.len(), 1, "flat(2) should be flagged as magic number");
    }

    #[test]
    fn test_flags_flat_with_depth_five() {
        let diags = lint("arr.flat(5);");
        assert_eq!(diags.len(), 1, "flat(5) should be flagged as magic number");
    }

    #[test]
    fn test_allows_flat_no_args() {
        let diags = lint("arr.flat();");
        assert!(
            diags.is_empty(),
            "flat() with no args should not be flagged"
        );
    }

    #[test]
    fn test_allows_flat_one() {
        let diags = lint("arr.flat(1);");
        assert!(
            diags.is_empty(),
            "flat(1) should not be flagged (default depth)"
        );
    }

    #[test]
    fn test_allows_flat_infinity() {
        let diags = lint("arr.flat(Infinity);");
        assert!(
            diags.is_empty(),
            "flat(Infinity) should not be flagged (common pattern)"
        );
    }

    #[test]
    fn test_allows_flat_variable() {
        let diags = lint("arr.flat(depth);");
        assert!(
            diags.is_empty(),
            "flat(variable) should not be flagged (not a magic number)"
        );
    }

    #[test]
    fn test_allows_non_flat_call() {
        let diags = lint("arr.map(x => x);");
        assert!(diags.is_empty(), "non-flat calls should not be flagged");
    }
}
