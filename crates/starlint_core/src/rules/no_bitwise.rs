//! Rule: `no-bitwise`
//!
//! Disallow bitwise operators. Bitwise operators are rarely used in
//! JavaScript and are often typos for logical operators (e.g. `&` vs `&&`).

use oxc_ast::AstKind;
use oxc_ast::ast::BinaryOperator;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags bitwise operators.
#[derive(Debug)]
pub struct NoBitwise;

impl NativeRule for NoBitwise {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-bitwise".to_owned(),
            description: "Disallow bitwise operators".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::BinaryExpression, AstType::UnaryExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::BinaryExpression(expr) => {
                if is_bitwise_binary(expr.operator) {
                    ctx.report(Diagnostic {
                        rule_name: "no-bitwise".to_owned(),
                        message: "Unexpected use of bitwise operator".to_owned(),
                        span: Span::new(expr.span.start, expr.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            AstKind::UnaryExpression(expr) => {
                if expr.operator == oxc_ast::ast::UnaryOperator::BitwiseNot {
                    ctx.report(Diagnostic {
                        rule_name: "no-bitwise".to_owned(),
                        message: "Unexpected use of bitwise operator `~`".to_owned(),
                        span: Span::new(expr.span.start, expr.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            _ => {}
        }
    }
}

/// Check if a binary operator is a bitwise operator.
const fn is_bitwise_binary(op: BinaryOperator) -> bool {
    matches!(
        op,
        BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseOR
            | BinaryOperator::BitwiseXOR
            | BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::ShiftRightZeroFill
    )
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoBitwise)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_bitwise_and() {
        let diags = lint("var x = a & b;");
        assert_eq!(diags.len(), 1, "bitwise AND should be flagged");
    }

    #[test]
    fn test_flags_bitwise_or() {
        let diags = lint("var x = a | b;");
        assert_eq!(diags.len(), 1, "bitwise OR should be flagged");
    }

    #[test]
    fn test_flags_bitwise_not() {
        let diags = lint("var x = ~a;");
        assert_eq!(diags.len(), 1, "bitwise NOT should be flagged");
    }

    #[test]
    fn test_allows_logical_and() {
        let diags = lint("var x = a && b;");
        assert!(diags.is_empty(), "logical AND should not be flagged");
    }

    #[test]
    fn test_allows_logical_or() {
        let diags = lint("var x = a || b;");
        assert!(diags.is_empty(), "logical OR should not be flagged");
    }
}
