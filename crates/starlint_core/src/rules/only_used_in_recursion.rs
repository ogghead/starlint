//! Rule: `only-used-in-recursion`
//!
//! Flag function parameters that are only passed through to recursive calls
//! at the same argument position, and never used in any other expression.
//! Such parameters contribute nothing to the computation and should be removed.
//!
//! This is a simplified heuristic that checks named function declarations.
//! It uses source-text analysis to count how a parameter identifier is used
//! inside the function body.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags function parameters that are only used in recursive calls.
#[derive(Debug)]
pub struct OnlyUsedInRecursion;

impl LintRule for OnlyUsedInRecursion {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "only-used-in-recursion".to_owned(),
            description:
                "Flag parameters only passed through to recursive calls at the same position"
                    .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::Function])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::Function(func) = node else {
            return;
        };

        // Only check named function declarations.
        let func_name = match func.id {
            Some(id) => match ctx.node(id) {
                Some(AstNode::BindingIdentifier(bi)) => bi.name.clone(),
                _ => return,
            },
            None => return,
        };

        let Some(body_id) = func.body else {
            return;
        };
        let Some(AstNode::FunctionBody(body)) = ctx.node(body_id) else {
            return;
        };
        let body_span = body.span;
        let body_stmt_ids: Vec<NodeId> = body.statements.to_vec();

        let param_info = collect_param_info(&func.params, ctx);
        if param_info.is_empty() {
            return;
        }

        // Collect all recursive call argument spans from the function body.
        let mut recursive_call_args: Vec<Box<[Span]>> = Vec::new();
        collect_recursive_call_args(&body_stmt_ids, &func_name, &mut recursive_call_args, ctx);

        if recursive_call_args.is_empty() {
            return;
        }

        // For each parameter, check if it is ONLY used as a same-position
        // argument in recursive calls and nowhere else.
        let flagged_params = {
            let source = ctx.source_text();
            let body_start = usize::try_from(body_span.start).unwrap_or(0);
            let body_end = usize::try_from(body_span.end).unwrap_or(source.len());
            let body_source = source.get(body_start..body_end).unwrap_or("");

            let mut flagged: Vec<(String, Span)> = Vec::new();

            for (param_idx, param_name, param_span) in &param_info {
                let total_uses = count_identifier_occurrences(body_source, param_name);
                if total_uses == 0 {
                    // Parameter is completely unused — a different rule handles that.
                    continue;
                }

                let recursive_uses = count_recursive_pass_through(
                    &recursive_call_args,
                    param_name,
                    *param_idx,
                    source,
                );

                if recursive_uses > 0 && recursive_uses == total_uses {
                    flagged.push((param_name.clone(), *param_span));
                }
            }
            flagged
        };

        for (param_name, param_span) in &flagged_params {
            ctx.report(Diagnostic {
                rule_name: "only-used-in-recursion".to_owned(),
                message: format!(
                    "Parameter `{param_name}` is only passed through to the recursive call at the same position"
                ),
                span: *param_span,
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Extract (index, name, span) for each simple binding-identifier parameter.
fn collect_param_info(params: &[NodeId], ctx: &LintContext<'_>) -> Vec<(usize, String, Span)> {
    let mut result = Vec::new();
    for (idx, param_id) in params.iter().enumerate() {
        if let Some(AstNode::BindingIdentifier(id)) = ctx.node(*param_id) {
            result.push((idx, id.name.clone(), Span::new(id.span.start, id.span.end)));
        }
    }
    result
}

/// Recursively walk statements and collect argument spans of recursive calls
/// whose callee matches `func_name`.
fn collect_recursive_call_args(
    stmt_ids: &[NodeId],
    func_name: &str,
    out: &mut Vec<Box<[Span]>>,
    ctx: &LintContext<'_>,
) {
    for stmt_id in stmt_ids {
        collect_recursive_calls_in_statement(*stmt_id, func_name, out, ctx);
    }
}

/// Walk a single statement for recursive calls.
fn collect_recursive_calls_in_statement(
    stmt_id: NodeId,
    func_name: &str,
    out: &mut Vec<Box<[Span]>>,
    ctx: &LintContext<'_>,
) {
    match ctx.node(stmt_id) {
        Some(AstNode::ExpressionStatement(expr_stmt)) => {
            collect_recursive_calls_in_expr(expr_stmt.expression, func_name, out, ctx);
        }
        Some(AstNode::ReturnStatement(ret)) => {
            if let Some(arg) = ret.argument {
                collect_recursive_calls_in_expr(arg, func_name, out, ctx);
            }
        }
        Some(AstNode::BlockStatement(block)) => {
            let body_ids: Vec<NodeId> = block.body.to_vec();
            collect_recursive_call_args(&body_ids, func_name, out, ctx);
        }
        Some(AstNode::IfStatement(if_stmt)) => {
            collect_recursive_calls_in_expr(if_stmt.test, func_name, out, ctx);
            collect_recursive_calls_in_statement(if_stmt.consequent, func_name, out, ctx);
            if let Some(alt) = if_stmt.alternate {
                collect_recursive_calls_in_statement(alt, func_name, out, ctx);
            }
        }
        Some(AstNode::VariableDeclaration(decl)) => {
            for decl_id in &decl.declarations {
                if let Some(AstNode::VariableDeclarator(declarator)) = ctx.node(*decl_id) {
                    if let Some(init) = declarator.init {
                        collect_recursive_calls_in_expr(init, func_name, out, ctx);
                    }
                }
            }
        }
        _ => {}
    }
}

/// Walk an expression tree for call expressions targeting `func_name`.
/// Instead of collecting references to `CallExpression` nodes, we collect
/// the argument spans as owned data.
fn collect_recursive_calls_in_expr(
    expr_id: NodeId,
    func_name: &str,
    out: &mut Vec<Box<[Span]>>,
    ctx: &LintContext<'_>,
) {
    match ctx.node(expr_id) {
        Some(AstNode::CallExpression(call)) => {
            if let Some(AstNode::IdentifierReference(ident)) = ctx.node(call.callee) {
                if ident.name.as_str() == func_name {
                    // Collect argument spans
                    let arg_spans: Vec<Span> = call
                        .arguments
                        .iter()
                        .map(|arg_id| {
                            ctx.node(*arg_id).map_or(Span::new(0, 0), |n| {
                                let s = n.span();
                                Span::new(s.start, s.end)
                            })
                        })
                        .collect();
                    out.push(arg_spans.into_boxed_slice());
                }
            }
        }
        Some(AstNode::BinaryExpression(bin)) => {
            collect_recursive_calls_in_expr(bin.left, func_name, out, ctx);
            collect_recursive_calls_in_expr(bin.right, func_name, out, ctx);
        }
        Some(AstNode::ConditionalExpression(cond)) => {
            collect_recursive_calls_in_expr(cond.test, func_name, out, ctx);
            collect_recursive_calls_in_expr(cond.consequent, func_name, out, ctx);
            collect_recursive_calls_in_expr(cond.alternate, func_name, out, ctx);
        }
        Some(AstNode::LogicalExpression(logic)) => {
            collect_recursive_calls_in_expr(logic.left, func_name, out, ctx);
            collect_recursive_calls_in_expr(logic.right, func_name, out, ctx);
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
    calls: &[Box<[Span]>],
    param_name: &str,
    param_idx: usize,
    source: &str,
) -> u32 {
    let mut count: u32 = 0;
    for arg_spans in calls {
        if let Some(arg_span) = arg_spans.get(param_idx) {
            let arg_start = usize::try_from(arg_span.start).unwrap_or(0);
            let arg_end = usize::try_from(arg_span.end).unwrap_or(0);
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

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(OnlyUsedInRecursion)];
        lint_source(source, "test.js", &rules)
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
