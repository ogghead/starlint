//! Rule: `no-constant-condition`
//!
//! Disallow constant expressions in conditions. A condition that always evaluates
//! to the same value is almost certainly a bug or dead code.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags conditions that are always truthy or falsy due to being a literal value.
#[derive(Debug)]
pub struct NoConstantCondition;

/// Returns `true` if the expression is a literal (boolean, numeric, null, string,
/// or a template literal with no interpolations).
fn is_constant_expression(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::BooleanLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::StringLiteral(_) => true,
        Expression::TemplateLiteral(tpl) => tpl.expressions.is_empty(),
        _ => false,
    }
}

/// Determine the JS truthiness of a constant literal expression.
///
/// Returns `Some(true)` for truthy literals, `Some(false)` for falsy, `None` if
/// truthiness cannot be determined.
fn is_truthy_literal(expr: &Expression<'_>) -> Option<bool> {
    match expr {
        Expression::BooleanLiteral(lit) => Some(lit.value),
        Expression::NumericLiteral(lit) => Some(lit.value != 0.0 && !lit.value.is_nan()),
        Expression::NullLiteral(_) => Some(false),
        Expression::StringLiteral(lit) => Some(!lit.value.is_empty()),
        Expression::TemplateLiteral(tpl) => {
            if !tpl.expressions.is_empty() {
                return None;
            }
            // Empty template `` is falsy (empty string), non-empty is truthy.
            let is_empty = tpl.quasis.iter().all(|q| q.value.raw.is_empty());
            Some(!is_empty)
        }
        _ => None,
    }
}

impl NativeRule for NoConstantCondition {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-constant-condition".to_owned(),
            description: "Disallow constant expressions in conditions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    #[allow(clippy::too_many_lines)] // Five AstKind arms with similar structure
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
                    let source = ctx.source_text();
                    let fix = is_truthy_literal(&expr.test).and_then(|truthy| {
                        let branch = if truthy {
                            &expr.consequent
                        } else {
                            &expr.alternate
                        };
                        let branch_span = branch.span();
                        let start = usize::try_from(branch_span.start).ok()?;
                        let end = usize::try_from(branch_span.end).ok()?;
                        let branch_text = source.get(start..end)?;
                        Some(Fix {
                            message: format!(
                                "Replace with {} branch",
                                if truthy { "consequent" } else { "alternate" }
                            ),
                            edits: vec![Edit {
                                span: Span::new(expr.span.start, expr.span.end),
                                replacement: branch_text.to_owned(),
                            }],
                        })
                    });
                    ctx.report(Diagnostic {
                        rule_name: "no-constant-condition".to_owned(),
                        message: "Unexpected constant condition in ternary expression".to_owned(),
                        span: Span::new(expr.span.start, expr.span.end),
                        severity: Severity::Error,
                        help: Some(
                            "Replace the constant condition with a dynamic expression".to_owned(),
                        ),
                        fix,
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
    fn test_flags_template_literal_no_interpolation() {
        let allocator = Allocator::default();
        let source = "if (`constant`) {}";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConstantCondition)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(
                diags.len(),
                1,
                "template literal without interpolation is constant"
            );
        }
    }

    #[test]
    fn test_allows_template_literal_with_interpolation() {
        let allocator = Allocator::default();
        let source = "if (`hello ${x}`) {}";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConstantCondition)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert!(
                diags.is_empty(),
                "template literal with interpolation is not constant"
            );
        }
    }

    #[test]
    fn test_ternary_true_fix_replaces_with_consequent() {
        let allocator = Allocator::default();
        let source = "var r = true ? 1 : 2;";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConstantCondition)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 1);
            let fix = diags.first().and_then(|d| d.fix.as_ref());
            assert!(fix.is_some(), "ternary should have a fix");
            let edit = fix.and_then(|f| f.edits.first());
            assert_eq!(
                edit.map(|e| e.replacement.as_str()),
                Some("1"),
                "truthy condition should replace with consequent"
            );
        }
    }

    #[test]
    fn test_ternary_false_fix_replaces_with_alternate() {
        let allocator = Allocator::default();
        let source = "var r = false ? 1 : 2;";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConstantCondition)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 1);
            let fix = diags.first().and_then(|d| d.fix.as_ref());
            assert!(fix.is_some(), "ternary should have a fix");
            let edit = fix.and_then(|f| f.edits.first());
            assert_eq!(
                edit.map(|e| e.replacement.as_str()),
                Some("2"),
                "falsy condition should replace with alternate"
            );
        }
    }

    #[test]
    fn test_ternary_null_fix() {
        let allocator = Allocator::default();
        let source = "var r = null ? a : b;";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConstantCondition)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            let edit = diags
                .first()
                .and_then(|d| d.fix.as_ref())
                .and_then(|f| f.edits.first());
            assert_eq!(
                edit.map(|e| e.replacement.as_str()),
                Some("b"),
                "null is falsy, should replace with alternate"
            );
        }
    }

    #[test]
    fn test_if_statement_has_no_fix() {
        let allocator = Allocator::default();
        let source = "if (true) { x(); }";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConstantCondition)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 1, "should flag if(true)");
            assert!(
                diags.first().and_then(|d| d.fix.as_ref()).is_none(),
                "if statement should not have a fix"
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
