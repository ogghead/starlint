//! Rule: `typescript/strict-boolean-expressions`
//!
//! Disallow using non-boolean types in boolean contexts. Flags `if` statements
//! whose condition is an obvious non-boolean literal: a string literal, the
//! number `0`, or the empty string `""`. These are almost always mistakes — the
//! developer likely intended a comparison.
//!
//! Simplified syntax-only version — full checking requires type information.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/strict-boolean-expressions";

/// Flags `if` statements whose condition is a non-boolean literal value.
#[derive(Debug)]
pub struct StrictBooleanExpressions;

impl NativeRule for StrictBooleanExpressions {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow non-boolean types in boolean contexts".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::IfStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::IfStatement(if_stmt) = kind else {
            return;
        };

        if let Some(description) = non_boolean_literal_kind(&if_stmt.test) {
            ctx.report_warning(
                RULE_NAME,
                &format!(
                    "Unexpected {description} in boolean context — use an explicit comparison \
                     instead"
                ),
                Span::new(if_stmt.test.span().start, if_stmt.test.span().end),
            );
        }
    }
}

/// Check if an expression is a non-boolean literal that should not appear in a
/// boolean context.
///
/// Returns a human-readable description of the problematic literal, or `None`
/// if the expression is acceptable.
fn non_boolean_literal_kind(expr: &Expression<'_>) -> Option<&'static str> {
    match expr {
        Expression::StringLiteral(_) => Some("string literal"),
        Expression::NumericLiteral(lit) if lit.value == 0.0 => Some("numeric literal `0`"),
        Expression::NullLiteral(_) => Some("`null` literal"),
        Expression::Identifier(ident) if ident.name.as_str() == "undefined" => Some("`undefined`"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(StrictBooleanExpressions)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_string_literal_in_if() {
        let diags = lint(r#"if ("hello") { console.log("yes"); }"#);
        assert_eq!(
            diags.len(),
            1,
            "string literal in if condition should be flagged"
        );
    }

    #[test]
    fn test_flags_empty_string_in_if() {
        let diags = lint(r#"if ("") { console.log("yes"); }"#);
        assert_eq!(
            diags.len(),
            1,
            "empty string in if condition should be flagged"
        );
    }

    #[test]
    fn test_flags_zero_in_if() {
        let diags = lint("if (0) { console.log('yes'); }");
        assert_eq!(
            diags.len(),
            1,
            "numeric literal 0 in if condition should be flagged"
        );
    }

    #[test]
    fn test_allows_boolean_in_if() {
        let diags = lint("if (true) { console.log('yes'); }");
        assert!(
            diags.is_empty(),
            "boolean literal in if condition should not be flagged"
        );
    }

    #[test]
    fn test_allows_comparison_in_if() {
        let diags = lint("if (x > 0) { console.log('yes'); }");
        assert!(
            diags.is_empty(),
            "comparison expression in if condition should not be flagged"
        );
    }
}
