//! Rule: `typescript/no-mixed-enums`
//!
//! Flags enum declarations that mix string and number initializers. Mixed enums
//! are confusing because they behave inconsistently: number members get reverse
//! mappings while string members do not, and the resulting runtime object has
//! different shapes depending on the mix.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::UnaryOperator;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags enum declarations that mix string and number initializers.
#[derive(Debug)]
pub struct NoMixedEnums;

impl LintRule for NoMixedEnums {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-mixed-enums".to_owned(),
            description: "Disallow enums that mix string and number members".to_owned(),
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

        let mut has_string = false;
        let mut has_number = false;

        // Get the enum name from its id
        let enum_name = ctx
            .node(decl.id)
            .and_then(|n| match n {
                AstNode::BindingIdentifier(id) => Some(id.name.as_str()),
                _ => None,
            })
            .unwrap_or("unknown");

        for &member_id in &*decl.members {
            let Some(AstNode::TSEnumMember(member)) = ctx.node(member_id) else {
                continue;
            };

            let Some(init_id) = member.initializer else {
                // Members without initializers are implicitly numeric (auto-incremented).
                has_number = true;
                continue;
            };

            match classify_initializer(init_id, ctx) {
                InitializerKind::String => has_string = true,
                InitializerKind::Number => has_number = true,
                InitializerKind::Other => {
                    // Computed expressions — can't determine statically, skip.
                }
            }

            if has_string && has_number {
                ctx.report(Diagnostic {
                    rule_name: "typescript/no-mixed-enums".to_owned(),
                    message: format!("Enum `{enum_name}` mixes string and number members"),
                    span: Span::new(decl.span.start, decl.span.end),
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

/// Classification of an enum member initializer.
enum InitializerKind {
    /// The initializer is a string literal or template literal.
    String,
    /// The initializer is a numeric literal (including negated numbers).
    Number,
    /// The initializer is a computed expression that cannot be classified.
    Other,
}

/// Classify an enum member initializer expression as string, number, or other.
fn classify_initializer(init_id: NodeId, ctx: &LintContext<'_>) -> InitializerKind {
    match ctx.node(init_id) {
        Some(AstNode::StringLiteral(_) | AstNode::TemplateLiteral(_)) => InitializerKind::String,
        Some(AstNode::NumericLiteral(_)) => InitializerKind::Number,
        Some(AstNode::UnaryExpression(unary)) => {
            // Handle negative numbers like `-1`.
            if matches!(unary.operator, UnaryOperator::UnaryNegation)
                && matches!(ctx.node(unary.argument), Some(AstNode::NumericLiteral(_)))
            {
                InitializerKind::Number
            } else {
                InitializerKind::Other
            }
        }
        _ => InitializerKind::Other,
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoMixedEnums, "test.ts");

    #[test]
    fn test_flags_mixed_string_and_number() {
        let source = r#"enum Mixed { A = 0, B = "hello" }"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "enum mixing string and number members should be flagged"
        );
    }

    #[test]
    fn test_flags_implicit_number_with_string() {
        let source = r#"enum Mixed { A, B = "hello" }"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "implicit numeric member mixed with string should be flagged"
        );
    }

    #[test]
    fn test_allows_all_number_members() {
        let source = "enum Numbers { A = 0, B = 1, C = 2 }";
        let diags = lint(source);
        assert!(diags.is_empty(), "all-number enum should not be flagged");
    }

    #[test]
    fn test_allows_all_string_members() {
        let source = r#"enum Strings { A = "a", B = "b", C = "c" }"#;
        let diags = lint(source);
        assert!(diags.is_empty(), "all-string enum should not be flagged");
    }

    #[test]
    fn test_allows_auto_incremented_members() {
        let source = "enum Auto { A, B, C }";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "auto-incremented enum should not be flagged"
        );
    }
}
