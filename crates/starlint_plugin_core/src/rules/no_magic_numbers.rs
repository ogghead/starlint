//! Rule: `no-magic-numbers`
//!
//! Flag magic numbers -- numeric literals used directly instead of named
//! constants. Common values like 0, 1, -1, and 2 are allowed.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags numeric literals that are not in the allowed set `{0, 1, -1, 2}`.
#[derive(Debug)]
pub struct NoMagicNumbers;

/// Check if a float value is in the set of allowed non-magic numbers.
#[allow(clippy::float_cmp)]
fn is_allowed_value(value: f64) -> bool {
    value == 0.0 || value == 1.0 || value == -1.0 || value == 2.0
}

impl LintRule for NoMagicNumbers {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-magic-numbers".to_owned(),
            description: "Disallow magic numbers".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::NumericLiteral])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::NumericLiteral(lit) = node else {
            return;
        };

        if is_allowed_value(lit.value) {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "no-magic-numbers".to_owned(),
            message: format!("No magic number: `{}`", lit.raw.as_str()),
            span: Span::new(lit.span.start, lit.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    starlint_rule_framework::lint_rule_test!(NoMagicNumbers);

    #[test]
    fn test_flags_magic_number() {
        let diags = lint("const x = 42;");
        assert_eq!(diags.len(), 1, "42 is a magic number and should be flagged");
    }

    #[test]
    fn test_allows_zero() {
        let diags = lint("const x = 0;");
        assert!(diags.is_empty(), "0 should not be flagged");
    }

    #[test]
    fn test_allows_one() {
        let diags = lint("const x = 1;");
        assert!(diags.is_empty(), "1 should not be flagged");
    }

    #[test]
    fn test_allows_two() {
        let diags = lint("const x = 2;");
        assert!(diags.is_empty(), "2 should not be flagged");
    }

    #[test]
    fn test_flags_ten_in_loop() {
        let diags = lint("for (let i = 0; i < 10; i++) {}");
        assert_eq!(diags.len(), 1, "10 should be flagged as a magic number");
    }

    #[test]
    fn test_flags_large_number() {
        let diags = lint("const timeout = 3000;");
        assert_eq!(diags.len(), 1, "3000 should be flagged as a magic number");
    }
}
