//! Rule: `bad-bitwise-operator` (OXC)
//!
//! Catch likely typos where `|` was used instead of `||` or `&` instead of
//! `&&`. This flags bitwise operators used with boolean operands (comparisons
//! or boolean literals).

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags bitwise `|` / `&` when both operands look boolean.
#[derive(Debug)]
pub struct BadBitwiseOperator;

impl NativeRule for BadBitwiseOperator {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "bad-bitwise-operator".to_owned(),
            description: "Catch `|` vs `||` and `&` vs `&&` operator typos".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::BinaryExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::BinaryExpression(expr) = kind else {
            return;
        };

        let intended = match expr.operator {
            BinaryOperator::BitwiseOR => "||",
            BinaryOperator::BitwiseAnd => "&&",
            _ => return,
        };

        if !looks_boolean(&expr.left) || !looks_boolean(&expr.right) {
            return;
        }

        let actual = if intended == "||" { "|" } else { "&" };
        ctx.report_warning(
            "bad-bitwise-operator",
            &format!("Suspicious use of `{actual}` — did you mean `{intended}`?"),
            Span::new(expr.span.start, expr.span.end),
        );
    }
}

/// Heuristic: does this expression look like it produces a boolean?
fn looks_boolean(expr: &Expression<'_>) -> bool {
    match expr {
        // Boolean literals and logical expressions always produce booleans
        Expression::BooleanLiteral(_) | Expression::LogicalExpression(_) => true,
        // Comparisons produce booleans
        Expression::BinaryExpression(bin) => {
            bin.operator.is_equality() || bin.operator.is_compare()
        }
        // !x produces a boolean
        Expression::UnaryExpression(un) => un.operator == oxc_ast::ast::UnaryOperator::LogicalNot,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(BadBitwiseOperator)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_pipe_with_booleans() {
        let diags = lint("if (a > 1 | b > 2) {}");
        assert_eq!(
            diags.len(),
            1,
            "bitwise OR with boolean operands should be flagged"
        );
    }

    #[test]
    fn test_flags_ampersand_with_booleans() {
        let diags = lint("if (a === 1 & b === 2) {}");
        assert_eq!(
            diags.len(),
            1,
            "bitwise AND with boolean operands should be flagged"
        );
    }

    #[test]
    fn test_allows_bitwise_with_numbers() {
        let diags = lint("var n = a | b;");
        assert!(
            diags.is_empty(),
            "bitwise OR with non-boolean operands should not be flagged"
        );
    }

    #[test]
    fn test_allows_logical_or() {
        let diags = lint("if (a > 1 || b > 2) {}");
        assert!(diags.is_empty(), "logical OR should not be flagged");
    }

    #[test]
    fn test_flags_boolean_literal_pipe() {
        let diags = lint("var x = true | false;");
        assert_eq!(
            diags.len(),
            1,
            "bitwise OR with boolean literals should be flagged"
        );
    }
}
