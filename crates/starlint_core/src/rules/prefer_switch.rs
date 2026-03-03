//! Rule: `prefer-switch` (unicorn)
//!
//! Flags chains of `if`/`else if` that compare the same variable with `===`.
//! When 3+ conditions compare the same identifier with strict equality, a
//! `switch` statement is clearer.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression, Statement};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Minimum number of `===` branches on the same identifier before flagging.
const MIN_CASES: u32 = 3;

/// Flags if-else-if chains that could be replaced with a `switch` statement.
#[derive(Debug)]
pub struct PreferSwitch;

impl NativeRule for PreferSwitch {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-switch".to_owned(),
            description: "Prefer `switch` over multiple `===` comparisons on the same variable"
                .to_owned(),
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

        // Only trigger on the top-level `if` of a chain. If this `if` is itself
        // the `alternate` of a parent `if`, skip it to avoid duplicate reports.
        // We detect this by checking if the source text immediately before the
        // `if` keyword ends with `else`.
        if is_else_if_branch(if_stmt.span.start, ctx.source_text()) {
            return;
        }

        // Extract the identifier being compared in the first branch.
        let Some(first_ident) = strict_eq_identifier(&if_stmt.test) else {
            return;
        };

        // Walk the else-if chain, counting branches that compare the same identifier.
        let mut count: u32 = 1;
        let mut current_alt = &if_stmt.alternate;
        while let Some(alt) = current_alt {
            if let Statement::IfStatement(else_if) = alt {
                if let Some(ident) = strict_eq_identifier(&else_if.test) {
                    if ident == first_ident {
                        count = count.saturating_add(1);
                        current_alt = &else_if.alternate;
                        continue;
                    }
                }
            }
            break;
        }

        if count >= MIN_CASES {
            ctx.report_warning(
                "prefer-switch",
                &format!(
                    "Use a `switch` statement instead of {count} `if`/`else if` comparisons on `{first_ident}`"
                ),
                Span::new(if_stmt.span.start, if_stmt.span.end),
            );
        }
    }
}

/// Check if this `if` statement is the `alternate` of a parent `if`.
/// Looks backwards from the `if` keyword for the word `else`.
fn is_else_if_branch(if_start: u32, source: &str) -> bool {
    let start = usize::try_from(if_start).unwrap_or(0);
    // Walk backwards over whitespace to find `else`.
    let before = source.get(..start).unwrap_or("");
    let trimmed = before.trim_end();
    trimmed.ends_with("else")
}

/// If the expression is a `BinaryExpression` with `===` and one side is an
/// `Identifier`, return that identifier's name.
fn strict_eq_identifier<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    let Expression::BinaryExpression(bin) = expr else {
        return None;
    };

    if bin.operator != BinaryOperator::StrictEquality {
        return None;
    }

    // Check left side first, then right.
    if let Expression::Identifier(id) = &bin.left {
        return Some(id.name.as_str());
    }
    if let Expression::Identifier(id) = &bin.right {
        return Some(id.name.as_str());
    }

    None
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferSwitch)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_three_strict_equality_branches() {
        let diags = lint("if (x === 1) {} else if (x === 2) {} else if (x === 3) {}");
        assert!(
            !diags.is_empty(),
            "3+ strict equality branches on same variable should be flagged"
        );
    }

    #[test]
    fn test_flags_four_branches() {
        let diags = lint(
            "if (x === 'a') {} else if (x === 'b') {} else if (x === 'c') {} else if (x === 'd') {}",
        );
        assert!(
            !diags.is_empty(),
            "4 strict equality branches should be flagged"
        );
    }

    #[test]
    fn test_allows_only_two_branches() {
        let diags = lint("if (x === 1) {} else if (x === 2) {}");
        assert!(diags.is_empty(), "only 2 branches should not be flagged");
    }

    #[test]
    fn test_allows_different_variables() {
        let diags = lint("if (x === 1) {} else if (y === 2) {} else if (z === 3) {}");
        assert!(
            diags.is_empty(),
            "different variables should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_strict_equality() {
        let diags = lint("if (x == 1) {} else if (x == 2) {} else if (x == 3) {}");
        assert!(diags.is_empty(), "loose equality should not be flagged");
    }

    #[test]
    fn test_allows_mixed_operators() {
        let diags = lint("if (x === 1) {} else if (x > 2) {} else if (x === 3) {}");
        assert!(diags.is_empty(), "mixed operators should break the chain");
    }

    #[test]
    fn test_allows_simple_if() {
        let diags = lint("if (x === 1) {}");
        assert!(diags.is_empty(), "single if should not be flagged");
    }

    #[test]
    fn test_allows_if_else_no_chain() {
        let diags = lint("if (x === 1) {} else {}");
        assert!(
            diags.is_empty(),
            "if-else without chain should not be flagged"
        );
    }

    #[test]
    fn test_identifier_on_right_side() {
        let diags = lint("if (1 === x) {} else if (2 === x) {} else if (3 === x) {}");
        assert!(
            !diags.is_empty(),
            "identifier on right side of === should also be detected"
        );
    }
}
