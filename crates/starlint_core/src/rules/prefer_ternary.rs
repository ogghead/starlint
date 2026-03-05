//! Rule: `prefer-ternary`
//!
//! Prefer ternary expressions over simple `if`/`else` that both return or
//! both assign to the same variable. Ternary expressions are more concise
//! for these trivial patterns.

use oxc_ast::AstKind;
use oxc_ast::ast::{AssignmentOperator, AssignmentTarget, Expression, Statement};
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags simple `if`/`else` blocks that could be ternary expressions.
#[derive(Debug)]
pub struct PreferTernary;

impl NativeRule for PreferTernary {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-ternary".to_owned(),
            description: "Prefer ternary expressions over simple if/else assignments or returns"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::IfStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::IfStatement(if_stmt) = kind else {
            return;
        };

        // Must have an else branch
        if if_stmt.alternate.is_none() {
            return;
        }

        let consequent = unwrap_single_statement(&if_stmt.consequent);
        let alternate = if_stmt
            .alternate
            .as_ref()
            .and_then(|alt| unwrap_single_statement(alt));

        let (Some(cons_stmt), Some(alt_stmt)) = (consequent, alternate) else {
            return;
        };

        // Case 1: both branches are single return statements with arguments
        let both_return = matches!(
            (cons_stmt, alt_stmt),
            (Statement::ReturnStatement(c), Statement::ReturnStatement(a))
            if c.argument.is_some() && a.argument.is_some()
        );

        // Case 2: both branches are single assignment expressions to the same
        // variable with the plain `=` operator
        let both_assign_same = is_simple_assign(cons_stmt)
            .zip(is_simple_assign(alt_stmt))
            .is_some_and(|(left, right)| left == right);

        if !both_return && !both_assign_same {
            return;
        }

        let source = ctx.source_text();
        let cond_start = usize::try_from(if_stmt.test.span().start).unwrap_or(0);
        let cond_end = usize::try_from(if_stmt.test.span().end).unwrap_or(0);
        let cond_text = source.get(cond_start..cond_end).unwrap_or("");

        let fix = if both_return {
            build_return_ternary(source, cond_text, cons_stmt, alt_stmt)
        } else {
            build_assign_ternary(source, cond_text, cons_stmt, alt_stmt)
        };

        ctx.report(Diagnostic {
            rule_name: "prefer-ternary".to_owned(),
            message: "This `if`/`else` can be replaced with a ternary expression".to_owned(),
            span: Span::new(if_stmt.span.start, if_stmt.span.end),
            severity: Severity::Warning,
            help: Some("Use a ternary expression".to_owned()),
            fix: fix.map(|replacement| Fix {
                message: "Convert to ternary expression".to_owned(),
                edits: vec![Edit {
                    span: Span::new(if_stmt.span.start, if_stmt.span.end),
                    replacement,
                }],
            }),
            labels: vec![],
        });
    }
}

/// If the statement is a block with exactly one statement, return that
/// statement. If it is already a non-block statement, return it directly.
/// Returns `None` for blocks with zero or multiple statements.
fn unwrap_single_statement<'a>(stmt: &'a Statement<'a>) -> Option<&'a Statement<'a>> {
    match stmt {
        Statement::BlockStatement(block) => {
            if block.body.len() == 1 {
                block.body.first()
            } else {
                None
            }
        }
        other => Some(other),
    }
}

/// If the statement is an expression statement containing a plain `=`
/// assignment, return the assignment target name. Returns `None` otherwise.
fn is_simple_assign<'a>(stmt: &'a Statement<'a>) -> Option<&'a str> {
    let Statement::ExpressionStatement(expr_stmt) = stmt else {
        return None;
    };
    let Expression::AssignmentExpression(assign) = &expr_stmt.expression else {
        return None;
    };
    if assign.operator != AssignmentOperator::Assign {
        return None;
    }
    assignment_target_name(&assign.left)
}

/// Extract a simple identifier name from an assignment target.
fn assignment_target_name<'a>(target: &'a AssignmentTarget<'a>) -> Option<&'a str> {
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(ident) => Some(ident.name.as_str()),
        _ => None,
    }
}

/// Build `return cond ? cons_val : alt_val;`
fn build_return_ternary(
    source: &str,
    cond_text: &str,
    cons_stmt: &Statement<'_>,
    alt_stmt: &Statement<'_>,
) -> Option<String> {
    let Statement::ReturnStatement(cons_ret) = cons_stmt else {
        return None;
    };
    let Statement::ReturnStatement(alt_ret) = alt_stmt else {
        return None;
    };
    let cons_arg = cons_ret.argument.as_ref()?;
    let alt_arg = alt_ret.argument.as_ref()?;

    let if_start = usize::try_from(cons_arg.span().start).unwrap_or(0);
    let if_end = usize::try_from(cons_arg.span().end).unwrap_or(0);
    let else_start = usize::try_from(alt_arg.span().start).unwrap_or(0);
    let else_end = usize::try_from(alt_arg.span().end).unwrap_or(0);

    let if_val = source.get(if_start..if_end)?;
    let else_val = source.get(else_start..else_end)?;

    Some(format!("return {cond_text} ? {if_val} : {else_val};"))
}

/// Build `target = cond ? cons_val : alt_val;`
fn build_assign_ternary(
    source: &str,
    cond_text: &str,
    if_stmt: &Statement<'_>,
    else_stmt: &Statement<'_>,
) -> Option<String> {
    let Statement::ExpressionStatement(if_expr) = if_stmt else {
        return None;
    };
    let Expression::AssignmentExpression(if_assign) = &if_expr.expression else {
        return None;
    };
    let Statement::ExpressionStatement(else_expr) = else_stmt else {
        return None;
    };
    let Expression::AssignmentExpression(else_assign) = &else_expr.expression else {
        return None;
    };

    let target_name = assignment_target_name(&if_assign.left)?;

    let if_start = usize::try_from(if_assign.right.span().start).unwrap_or(0);
    let if_end = usize::try_from(if_assign.right.span().end).unwrap_or(0);
    let else_start = usize::try_from(else_assign.right.span().start).unwrap_or(0);
    let else_end = usize::try_from(else_assign.right.span().end).unwrap_or(0);

    let if_val = source.get(if_start..if_end)?;
    let else_val = source.get(else_start..else_end)?;

    Some(format!(
        "{target_name} = {cond_text} ? {if_val} : {else_val};"
    ))
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferTernary)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_simple_return() {
        let diags = lint("function f(x) { if (x) { return a; } else { return b; } }");
        assert_eq!(diags.len(), 1, "simple if/else return should be flagged");
    }

    #[test]
    fn test_allows_no_else() {
        let diags = lint("function f(x) { if (x) { return a; } }");
        assert!(diags.is_empty(), "if without else should not be flagged");
    }

    #[test]
    fn test_allows_multiple_statements_in_consequent() {
        let diags = lint("function f(x) { if (x) { foo(); return a; } else { return b; } }");
        assert!(
            diags.is_empty(),
            "multiple statements in if-block should not be flagged"
        );
    }

    #[test]
    fn test_flags_simple_assignment() {
        let diags = lint("var a; if (x) { a = 1; } else { a = 2; }");
        assert_eq!(
            diags.len(),
            1,
            "simple if/else assignment to same var should be flagged"
        );
    }

    #[test]
    fn test_allows_different_assignment_targets() {
        let diags = lint("if (x) { a = 1; } else { b = 2; }");
        assert!(
            diags.is_empty(),
            "assignment to different vars should not be flagged"
        );
    }

    #[test]
    fn test_allows_empty_return() {
        let diags = lint("function f(x) { if (x) { return; } else { return; } }");
        assert!(
            diags.is_empty(),
            "empty returns should not be flagged (no value to ternary-ize)"
        );
    }
}
