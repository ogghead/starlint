//! Rule: `typescript/no-unnecessary-condition`
//!
//! Disallow unnecessary conditions. Flags `if (true)`, `if (false)`, and
//! `while (true)` where the condition is a boolean literal and is therefore
//! always known at compile time.
//!
//! Simplified syntax-only version — full checking requires type information.
//! The full rule also flags conditions whose type is always truthy/falsy
//! after narrowing; this simplified version only detects boolean literal
//! constants.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/no-unnecessary-condition";

/// Flags `if` and `while` statements whose condition is a boolean literal.
#[derive(Debug)]
pub struct NoUnnecessaryCondition;

impl NativeRule for NoUnnecessaryCondition {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow unnecessary conditions (boolean literal in condition position)"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::IfStatement(stmt) => {
                if let Some(value) = boolean_literal_value(&stmt.test) {
                    let label = if value { "true" } else { "false" };
                    ctx.report_warning(
                        RULE_NAME,
                        &format!(
                            "Unnecessary condition — `if ({label})` is always {}, \
                             the branch is {}",
                            label,
                            if value { "always taken" } else { "dead code" },
                        ),
                        Span::new(stmt.span.start, stmt.span.end),
                    );
                }
            }
            AstKind::WhileStatement(stmt) => {
                if let Some(value) = boolean_literal_value(&stmt.test) {
                    let label = if value { "true" } else { "false" };
                    ctx.report_warning(
                        RULE_NAME,
                        &format!(
                            "Unnecessary condition — `while ({label})` is a constant \
                             loop condition"
                        ),
                        Span::new(stmt.span.start, stmt.span.end),
                    );
                }
            }
            _ => {}
        }
    }
}

/// If the expression is a `BooleanLiteral`, return its value.
fn boolean_literal_value(expr: &Expression<'_>) -> Option<bool> {
    if let Expression::BooleanLiteral(lit) = expr {
        Some(lit.value)
    } else {
        None
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnnecessaryCondition)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_if_true() {
        let diags = lint("if (true) { console.log('always'); }");
        assert_eq!(diags.len(), 1, "`if (true)` should be flagged");
    }

    #[test]
    fn test_flags_if_false() {
        let diags = lint("if (false) { console.log('never'); }");
        assert_eq!(diags.len(), 1, "`if (false)` should be flagged");
    }

    #[test]
    fn test_flags_while_true() {
        let diags = lint("while (true) { break; }");
        assert_eq!(diags.len(), 1, "`while (true)` should be flagged");
    }

    #[test]
    fn test_allows_dynamic_condition() {
        let diags = lint("const x = Math.random(); if (x > 0.5) { console.log('maybe'); }");
        assert!(
            diags.is_empty(),
            "dynamic condition should not be flagged"
        );
    }

    #[test]
    fn test_allows_variable_while_condition() {
        let diags = lint("let running = true; while (running) { running = false; }");
        assert!(
            diags.is_empty(),
            "variable in while condition should not be flagged"
        );
    }
}
