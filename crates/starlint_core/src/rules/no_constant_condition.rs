//! Rule: `no-constant-condition`
//!
//! Disallow constant expressions in conditions. A condition that always evaluates
//! to the same value is almost certainly a bug or dead code.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags conditions that are always truthy or falsy due to being a literal value.
#[derive(Debug)]
pub struct NoConstantCondition;

/// Returns `true` if the expression is a literal (boolean, numeric, null, or string).
const fn is_constant_expression(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::BooleanLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::StringLiteral(_)
    )
}

impl NativeRule for NoConstantCondition {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-constant-condition".to_owned(),
            description: "Disallow constant expressions in conditions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::IfStatement(stmt) => {
                if is_constant_expression(&stmt.test) {
                    ctx.report(Diagnostic {
                        rule_name: "no-constant-condition".to_owned(),
                        message: "Unexpected constant condition in `if` statement".to_owned(),
                        span: Span::new(stmt.span.start, stmt.span.end),
                        severity: Severity::Error,
                        help: Some(
                            "Replace the constant condition with a dynamic expression".to_owned(),
                        ),
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            AstKind::WhileStatement(stmt) => {
                if is_constant_expression(&stmt.test) {
                    ctx.report(Diagnostic {
                        rule_name: "no-constant-condition".to_owned(),
                        message: "Unexpected constant condition in `while` statement".to_owned(),
                        span: Span::new(stmt.span.start, stmt.span.end),
                        severity: Severity::Error,
                        help: Some(
                            "Replace the constant condition with a dynamic expression".to_owned(),
                        ),
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            AstKind::DoWhileStatement(stmt) => {
                if is_constant_expression(&stmt.test) {
                    ctx.report(Diagnostic {
                        rule_name: "no-constant-condition".to_owned(),
                        message: "Unexpected constant condition in `do-while` statement".to_owned(),
                        span: Span::new(stmt.span.start, stmt.span.end),
                        severity: Severity::Error,
                        help: Some(
                            "Replace the constant condition with a dynamic expression".to_owned(),
                        ),
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            AstKind::ForStatement(stmt) => {
                if let Some(test) = &stmt.test {
                    if is_constant_expression(test) {
                        ctx.report(Diagnostic {
                            rule_name: "no-constant-condition".to_owned(),
                            message: "Unexpected constant condition in `for` statement".to_owned(),
                            span: Span::new(stmt.span.start, stmt.span.end),
                            severity: Severity::Error,
                            help: Some(
                                "Replace the constant condition with a dynamic expression"
                                    .to_owned(),
                            ),
                            fix: None,
                            labels: vec![],
                        });
                    }
                }
            }
            AstKind::ConditionalExpression(expr) => {
                if is_constant_expression(&expr.test) {
                    ctx.report(Diagnostic {
                        rule_name: "no-constant-condition".to_owned(),
                        message: "Unexpected constant condition in ternary expression".to_owned(),
                        span: Span::new(expr.span.start, expr.span.end),
                        severity: Severity::Error,
                        help: Some(
                            "Replace the constant condition with a dynamic expression".to_owned(),
                        ),
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    #[test]
    fn test_flags_if_true() {
        let allocator = Allocator::default();
        let source = "if (true) {}";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConstantCondition)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 1, "should flag constant condition");
            assert_eq!(
                diags.first().map(|d| d.rule_name.as_str()),
                Some("no-constant-condition"),
                "rule name should match"
            );
        }
    }

    #[test]
    fn test_flags_while_false() {
        let allocator = Allocator::default();
        let source = "while (false) {}";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConstantCondition)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 1, "should flag constant condition");
            assert_eq!(
                diags.first().map(|d| d.rule_name.as_str()),
                Some("no-constant-condition"),
                "rule name should match"
            );
        }
    }

    #[test]
    fn test_allows_variable_condition() {
        let allocator = Allocator::default();
        let source = "if (x) {}";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConstantCondition)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert!(diags.is_empty(), "variable condition should not be flagged");
        }
    }

    #[test]
    fn test_flags_ternary_literal() {
        let allocator = Allocator::default();
        let source = "var r = true ? 1 : 2;";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConstantCondition)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 1, "should flag constant ternary condition");
            assert_eq!(
                diags.first().map(|d| d.rule_name.as_str()),
                Some("no-constant-condition"),
                "rule name should match"
            );
        }
    }

    #[test]
    fn test_flags_if_zero() {
        let allocator = Allocator::default();
        let source = "if (0) { doSomething(); }";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConstantCondition)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 1, "should flag numeric literal condition");
        }
    }

    #[test]
    fn test_flags_if_null() {
        let allocator = Allocator::default();
        let source = "if (null) { doSomething(); }";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConstantCondition)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 1, "should flag null literal condition");
        }
    }

    #[test]
    fn test_flags_if_string() {
        let allocator = Allocator::default();
        let source = r#"if ("yes") { doSomething(); }"#;
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConstantCondition)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 1, "should flag string literal condition");
        }
    }

    #[test]
    fn test_flags_do_while_constant() {
        let allocator = Allocator::default();
        let source = "do { x++; } while (true);";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConstantCondition)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 1, "should flag constant do-while condition");
        }
    }

    #[test]
    fn test_flags_for_constant_test() {
        let allocator = Allocator::default();
        let source = "for (let i = 0; true; i++) {}";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConstantCondition)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 1, "should flag constant for-loop test");
        }
    }

    #[test]
    fn test_allows_for_no_test() {
        let allocator = Allocator::default();
        let source = "for (;;) { break; }";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConstantCondition)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert!(
                diags.is_empty(),
                "for loop with no test should not be flagged"
            );
        }
    }

    #[test]
    fn test_allows_ternary_variable() {
        let allocator = Allocator::default();
        let source = "var r = x ? 1 : 2;";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConstantCondition)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert!(
                diags.is_empty(),
                "ternary with variable condition should not be flagged"
            );
        }
    }
}
