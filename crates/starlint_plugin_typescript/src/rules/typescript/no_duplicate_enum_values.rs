//! Rule: `typescript/no-duplicate-enum-values`
//!
//! Disallow duplicate enum member values. When multiple enum members share the
//! same initializer value (string or number literal), the later members silently
//! shadow earlier ones, which is almost always a mistake.

use std::collections::HashSet;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags enum declarations that contain members with duplicate initializer values.
#[derive(Debug)]
pub struct NoDuplicateEnumValues;

impl LintRule for NoDuplicateEnumValues {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-duplicate-enum-values".to_owned(),
            description: "Disallow duplicate enum member values".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TSEnumDeclaration])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::TSEnumDeclaration(decl) = node else {
            return;
        };

        let mut seen = HashSet::new();

        for member_id in &decl.members {
            let Some(AstNode::TSEnumMember(member)) = ctx.node(*member_id) else {
                continue;
            };

            let Some(init_id) = member.initializer else {
                // Auto-incremented members — no explicit value to check.
                continue;
            };

            let Some(value_key) = static_initializer_key(init_id, ctx) else {
                continue;
            };

            if !seen.insert(value_key.clone()) {
                ctx.report(Diagnostic {
                    rule_name: "typescript/no-duplicate-enum-values".to_owned(),
                    message: format!("Duplicate enum value `{value_key}`"),
                    span: Span::new(member.span.start, member.span.end),
                    severity: Severity::Error,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
        }
    }
}

/// Extract a comparable string key from an enum member initializer expression.
///
/// Returns `Some` for string and numeric literals, `None` for anything else
/// (computed expressions, identifiers, etc.).
fn static_initializer_key(node_id: NodeId, ctx: &LintContext<'_>) -> Option<String> {
    match ctx.node(node_id)? {
        AstNode::StringLiteral(lit) => Some(format!("\"{}\"", lit.value)),
        AstNode::NumericLiteral(lit) => Some(lit.value.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoDuplicateEnumValues, "test.ts");

    #[test]
    fn test_flags_duplicate_number_values() {
        let diags = lint("enum E { A = 1, B = 1 }");
        assert_eq!(
            diags.len(),
            1,
            "duplicate number enum values should be flagged"
        );
    }

    #[test]
    fn test_flags_duplicate_string_values() {
        let diags = lint(r#"enum E { A = "x", B = "x" }"#);
        assert_eq!(
            diags.len(),
            1,
            "duplicate string enum values should be flagged"
        );
    }

    #[test]
    fn test_allows_unique_values() {
        let diags = lint("enum E { A = 1, B = 2 }");
        assert!(diags.is_empty(), "unique enum values should not be flagged");
    }

    #[test]
    fn test_allows_auto_incremented() {
        let diags = lint("enum E { A, B }");
        assert!(
            diags.is_empty(),
            "auto-incremented enum members should not be flagged"
        );
    }
}
