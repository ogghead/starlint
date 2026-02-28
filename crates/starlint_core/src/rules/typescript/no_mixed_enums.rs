//! Rule: `typescript/no-mixed-enums`
//!
//! Flags enum declarations that mix string and number initializers. Mixed enums
//! are confusing because they behave inconsistently: number members get reverse
//! mappings while string members do not, and the resulting runtime object has
//! different shapes depending on the mix.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags enum declarations that mix string and number initializers.
#[derive(Debug)]
pub struct NoMixedEnums;

impl NativeRule for NoMixedEnums {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-mixed-enums".to_owned(),
            description: "Disallow enums that mix string and number members".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TSEnumDeclaration(decl) = kind else {
            return;
        };

        let mut has_string = false;
        let mut has_number = false;

        for member in &decl.body.members {
            let Some(ref init) = member.initializer else {
                // Members without initializers are implicitly numeric (auto-incremented).
                has_number = true;
                continue;
            };

            match classify_initializer(init) {
                InitializerKind::String => has_string = true,
                InitializerKind::Number => has_number = true,
                InitializerKind::Other => {
                    // Computed expressions — can't determine statically, skip.
                }
            }

            if has_string && has_number {
                let name = decl.id.name.as_str();
                ctx.report_error(
                    "typescript/no-mixed-enums",
                    &format!("Enum `{name}` mixes string and number members"),
                    Span::new(decl.span.start, decl.span.end),
                );
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
fn classify_initializer(expr: &Expression<'_>) -> InitializerKind {
    match expr {
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => InitializerKind::String,
        Expression::NumericLiteral(_) => InitializerKind::Number,
        Expression::UnaryExpression(unary) => {
            // Handle negative numbers like `-1`.
            if matches!(unary.operator, oxc_ast::ast::UnaryOperator::UnaryNegation)
                && matches!(unary.argument, Expression::NumericLiteral(_))
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code as TypeScript.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoMixedEnums)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

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
