//! Rule: `only-used-in-recursion`
//!
//! Flag function parameters that are only passed through to recursive calls
//! at the same argument position, and never used in any other expression.
//! Such parameters contribute nothing to the computation and should be removed.
//!
//! This is a simplified heuristic that checks named function declarations.
//! It uses source-text analysis to count how a parameter identifier is used
//! inside the function body.

use oxc_ast::AstKind;
use oxc_ast::ast::{BindingPattern, CallExpression, Expression, FormalParameters, Statement};
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags function parameters that are only used in recursive calls.
#[derive(Debug)]
pub struct OnlyUsedInRecursion;

impl NativeRule for OnlyUsedInRecursion {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "only-used-in-recursion".to_owned(),
            description:
                "Flag parameters only passed through to recursive calls at the same position"
                    .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::Function])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::Function(func) = kind else {
            return;
        };

        // Only check named function declarations.
        let Some(func_id) = &func.id else {
            return;
        };
        let func_name = func_id.name.as_str();

        let Some(body) = &func.body else {
            return;
        };

        let params = &func.params;
        let param_info = collect_param_info(params);
        if param_info.is_empty() {
            return;
        }

        // Collect all recursive calls to the same function name.
        let mut recursive_calls: Vec<&CallExpression<'_>> = Vec::new();
        collect_recursive_calls(&body.statements, func_name, &mut recursive_calls);

        if recursive_calls.is_empty() {
            return;
        }

        // For each parameter, check if it is ONLY used as a same-position
        // argument in recursive calls and nowhere else.
        //
        // Collect flagged params first, then report — avoids borrowing `ctx`
        // immutably (via `source_text()`) and mutably (via `report_warning()`)
        // at the same time.
        let flagged_params = {
            let source = ctx.source_text();
            let body_start = usize::try_from(body.span.start).unwrap_or(0);
            let body_end = usize::try_from(body.span.end).unwrap_or(source.len());
            let body_source = source.get(body_start..body_end).unwrap_or("");

            let mut flagged: Vec<(String, Span)> = Vec::new();

            for (param_idx, param_name, param_span) in &param_info {
                let total_uses = count_identifier_occurrences(body_source, param_name);
                if total_uses == 0 {
                    // Parameter is completely unused — a different rule handles that.
                    continue;
                }

                let recursive_uses =
                    count_recursive_pass_through(&recursive_calls, param_name, *param_idx, source);

                if recursive_uses > 0 && recursive_uses == total_uses {
                    flagged.push((param_name.clone(), *param_span));
                }
            }
            flagged
        };

        for (param_name, param_span) in &flagged_params {
            ctx.report_warning(
                "only-used-in-recursion",
                &format!(
                    "Parameter `{param_name}` is only passed through to the recursive call at the same position"
                ),
                *param_span,
            );
        }
    }
}

/// Extract (index, name, span) for each simple binding-identifier parameter.
fn collect_param_info(params: &FormalParameters<'_>) -> Vec<(usize, String, Span)> {
    let mut result = Vec::new();
    for (idx, param) in params.items.iter().enumerate() {
        if let BindingPattern::BindingIdentifier(id) = &param.pattern {
            result.push((
                idx,
                id.name.to_string(),
                Span::new(id.span.start, id.span.end),
            ));
        }
    }
    result
}

/// Recursively walk statements and collect `CallExpression` nodes whose
/// callee is a plain identifier matching `func_name`.
fn collect_recursive_calls<'a>(
    stmts: &'a [Statement<'a>],
    func_name: &str,
    out: &mut Vec<&'a CallExpression<'a>>,
) {
    for stmt in stmts {
        collect_recursive_calls_in_statement(stmt, func_name, out);
    }
}

/// Walk a single statement for recursive calls.
fn collect_recursive_calls_in_statement<'a>(
    stmt: &'a Statement<'a>,
    func_name: &str,
    out: &mut Vec<&'a CallExpression<'a>>,
) {
    match stmt {
        Statement::ExpressionStatement(expr_stmt) => {
            collect_recursive_calls_in_expr(&expr_stmt.expression, func_name, out);
        }
        Statement::ReturnStatement(ret) => {
            if let Some(arg) = &ret.argument {
                collect_recursive_calls_in_expr(arg, func_name, out);
            }
        }
        Statement::BlockStatement(block) => {
            collect_recursive_calls(&block.body, func_name, out);
        }
        Statement::IfStatement(if_stmt) => {
            collect_recursive_calls_in_expr(&if_stmt.test, func_name, out);
            collect_recursive_calls_in_statement(&if_stmt.consequent, func_name, out);
            if let Some(alt) = &if_stmt.alternate {
                collect_recursive_calls_in_statement(alt, func_name, out);
            }
        }
        Statement::VariableDeclaration(decl) => {
            for declarator in &decl.declarations {
                if let Some(init) = &declarator.init {
                    collect_recursive_calls_in_expr(init, func_name, out);
                }
            }
        }
        _ => {}
    }
}

/// Walk an expression tree for call expressions targeting `func_name`.
fn collect_recursive_calls_in_expr<'a>(
    expr: &'a Expression<'a>,
    func_name: &str,
    out: &mut Vec<&'a CallExpression<'a>>,
) {
    match expr {
        Expression::CallExpression(call) => {
            if let Expression::Identifier(ident) = &call.callee {
                if ident.name.as_str() == func_name {
                    out.push(call);
                }
            }
        }
        Expression::BinaryExpression(bin) => {
            collect_recursive_calls_in_expr(&bin.left, func_name, out);
            collect_recursive_calls_in_expr(&bin.right, func_name, out);
        }
        Expression::ConditionalExpression(cond) => {
            collect_recursive_calls_in_expr(&cond.test, func_name, out);
            collect_recursive_calls_in_expr(&cond.consequent, func_name, out);
            collect_recursive_calls_in_expr(&cond.alternate, func_name, out);
        }
        Expression::LogicalExpression(logic) => {
            collect_recursive_calls_in_expr(&logic.left, func_name, out);
            collect_recursive_calls_in_expr(&logic.right, func_name, out);
        }
        Expression::ParenthesizedExpression(paren) => {
            collect_recursive_calls_in_expr(&paren.expression, func_name, out);
        }
        _ => {}
    }
}

/// Count occurrences of an identifier in source text (word-boundary aware).
///
/// Uses a simple scan: matches `param_name` surrounded by non-identifier chars.
fn count_identifier_occurrences(source: &str, name: &str) -> u32 {
    let mut count: u32 = 0;
    let name_bytes = name.as_bytes();
    let name_len = name_bytes.len();
    let src_bytes = source.as_bytes();
    let src_len = src_bytes.len();

    if name_len == 0 || src_len < name_len {
        return 0;
    }

    let mut pos: usize = 0;
    while pos.saturating_add(name_len) <= src_len {
        if src_bytes.get(pos..pos.saturating_add(name_len)) == Some(name_bytes) {
            // Check boundaries: preceding char must not be identifier-like.
            let before_ok = pos == 0
                || src_bytes
                    .get(pos.saturating_sub(1))
                    .is_none_or(|b| !is_ident_char(*b));
            let after_ok = pos.saturating_add(name_len) >= src_len
                || src_bytes
                    .get(pos.saturating_add(name_len))
                    .is_none_or(|b| !is_ident_char(*b));

            if before_ok && after_ok {
                count = count.saturating_add(1);
            }
        }
        pos = pos.saturating_add(1);
    }

    count
}

/// Check whether a byte is an identifier character (alphanumeric or `_` or `$`).
const fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

/// Count how many recursive calls pass `param_name` at position `param_idx`
/// as a plain identifier argument.
fn count_recursive_pass_through(
    calls: &[&CallExpression<'_>],
    param_name: &str,
    param_idx: usize,
    source: &str,
) -> u32 {
    let mut count: u32 = 0;
    for call in calls {
        if let Some(arg) = call.arguments.get(param_idx) {
            let arg_start = usize::try_from(arg.span().start).unwrap_or(0);
            let arg_end = usize::try_from(arg.span().end).unwrap_or(0);
            let arg_text = source.get(arg_start..arg_end).unwrap_or("");
            if arg_text == param_name {
                count = count.saturating_add(1);
            }
        }
    }
    count
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(OnlyUsedInRecursion)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_pass_through_param() {
        // `b` is only used as second argument to recursive call foo(a - 1, b).
        let diags = lint("function foo(a, b) { if (a === 0) return 1; return foo(a - 1, b); }");
        assert_eq!(
            diags.len(),
            1,
            "parameter b should be flagged as only used in recursion"
        );
        assert!(
            diags.first().is_some_and(|d| d.message.contains('b')),
            "diagnostic should mention parameter b"
        );
    }

    #[test]
    fn test_allows_param_used_in_expression() {
        let diags = lint("function foo(a) { return a + 1; }");
        assert!(
            diags.is_empty(),
            "parameter used in expression should not be flagged"
        );
    }

    #[test]
    fn test_allows_param_used_in_both_expression_and_recursion() {
        let diags = lint("function foo(a, b) { return a + b + foo(a - 1, b); }");
        assert!(
            diags.is_empty(),
            "parameters used in expression and recursion should not be flagged"
        );
    }

    #[test]
    fn test_flags_only_recursive_call() {
        // `a` is only passed to foo(a) — flagged.
        let diags = lint("function foo(a) { return foo(a); }");
        assert_eq!(
            diags.len(),
            1,
            "parameter only in recursive call should be flagged"
        );
    }

    #[test]
    fn test_allows_no_recursion() {
        let diags = lint("function foo(a) { return a; }");
        assert!(
            diags.is_empty(),
            "non-recursive function should not be flagged"
        );
    }
}
