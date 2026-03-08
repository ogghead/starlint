//! Rule: `typescript/explicit-module-boundary-types`
//!
//! Require explicit types on exported functions and class methods. Public API
//! boundaries should have explicit types for documentation and stability.
//! Without explicit types, internal refactoring can accidentally change the
//! public contract.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/explicit-module-boundary-types";

/// Flags exported functions and class methods that lack explicit return type
/// or parameter type annotations.
#[derive(Debug)]
pub struct ExplicitModuleBoundaryTypes;

impl LintRule for ExplicitModuleBoundaryTypes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require explicit types on exported functions and class methods"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::ExportDefaultDeclaration,
            AstNodeType::ExportNamedDeclaration,
        ])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::ExportNamedDeclaration(decl) => {
                if let Some(declaration_id) = decl.declaration {
                    check_declaration(declaration_id, ctx);
                }
            }
            AstNode::ExportDefaultDeclaration(decl) => {
                check_default_declaration(decl.declaration, decl.span.start, decl.span.end, ctx);
            }
            _ => {}
        }
    }
}

/// Check a named export declaration for missing type annotations.
/// Uses source text heuristic since `FunctionNode` has no `return_type` field.
fn check_declaration(decl_id: NodeId, ctx: &mut LintContext<'_>) {
    // Extract all needed data from the borrowed node before calling ctx.report()
    let (func_span, func_body, func_params) = {
        let Some(AstNode::Function(func)) = ctx.node(decl_id) else {
            return;
        };
        // Skip functions without a body (ambient declarations)
        if func.body.is_none() {
            return;
        }
        (func.span, func.body, func.params.to_vec())
    };

    // Check for return type using source text heuristic
    let has_return_type = has_return_type_annotation(func_span, func_body, ctx);

    if !has_return_type {
        ctx.report(Diagnostic {
            rule_name: RULE_NAME.to_owned(),
            message: "Exported function missing explicit return type".to_owned(),
            span: Span::new(func_span.start, func_span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }

    // Check each parameter for a type annotation using source text
    check_params_for_types(&func_params, ctx);
}

/// Check a default export declaration for missing type annotations.
fn check_default_declaration(
    decl_id: NodeId,
    span_start: u32,
    span_end: u32,
    ctx: &mut LintContext<'_>,
) {
    // Extract data from the node before mutably borrowing ctx
    enum DeclKind {
        Function {
            func_span: starlint_ast::types::Span,
            func_body: Option<NodeId>,
            func_params: Vec<NodeId>,
        },
        Arrow {
            arrow_span: starlint_ast::types::Span,
            arrow_body: NodeId,
            arrow_params: Vec<NodeId>,
        },
        Other,
    }

    let kind = match ctx.node(decl_id) {
        Some(AstNode::Function(func)) => {
            if func.body.is_none() {
                return;
            }
            DeclKind::Function {
                func_span: func.span,
                func_body: func.body,
                func_params: func.params.to_vec(),
            }
        }
        Some(AstNode::ArrowFunctionExpression(arrow)) => DeclKind::Arrow {
            arrow_span: arrow.span,
            arrow_body: arrow.body,
            arrow_params: arrow.params.to_vec(),
        },
        _ => DeclKind::Other,
    };

    match kind {
        DeclKind::Function {
            func_span,
            func_body,
            func_params,
        } => {
            let has_return_type = has_return_type_annotation(func_span, func_body, ctx);

            if !has_return_type {
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: "Exported function missing explicit return type".to_owned(),
                    span: Span::new(func_span.start, func_span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }

            check_params_for_types(&func_params, ctx);
        }
        DeclKind::Arrow {
            arrow_span,
            arrow_body,
            arrow_params,
        } => {
            let source = ctx.source_text();
            // For arrows, check between params end and body start
            let body_span = ctx.node(arrow_body).map_or(
                starlint_ast::types::Span::EMPTY,
                starlint_ast::AstNode::span,
            );
            let params_end = arrow_params
                .last()
                .and_then(|&id| ctx.node(id))
                .map_or(arrow_span.start, |n| n.span().end);
            let region_start = usize::try_from(params_end).unwrap_or(0);
            let region_end = usize::try_from(body_span.start).unwrap_or(0);
            let between = source.get(region_start..region_end).unwrap_or("");
            // Strip the `=>` and check for `:` before it
            let has_return_type = if let Some(arrow_pos) = between.find("=>") {
                between.get(..arrow_pos).is_some_and(|s| s.contains(':'))
            } else {
                between.contains(':')
            };

            if !has_return_type {
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: "Exported arrow function missing explicit return type".to_owned(),
                    span: Span::new(span_start, span_end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }

            check_params_for_types(&arrow_params, ctx);
        }
        DeclKind::Other => {}
    }
}

/// Check if a function has a return type annotation using source text heuristic.
fn has_return_type_annotation(
    func_span: starlint_ast::types::Span,
    body_id: Option<NodeId>,
    ctx: &LintContext<'_>,
) -> bool {
    let Some(body_node_id) = body_id else {
        return false;
    };
    let body_span = ctx.node(body_node_id).map_or(
        starlint_ast::types::Span::EMPTY,
        starlint_ast::AstNode::span,
    );
    let source = ctx.source_text();
    // Look for `:` between the last `)` before body and `{` of body
    let search_start = usize::try_from(func_span.start).unwrap_or(0);
    let search_end = usize::try_from(body_span.start).unwrap_or(0);
    let between = source.get(search_start..search_end).unwrap_or("");
    // Find the last `)` and check if there's a `:` after it
    if let Some(paren_pos) = between.rfind(')') {
        between.get(paren_pos..).is_some_and(|s| s.contains(':'))
    } else {
        false
    }
}

/// Check function parameters for type annotations using source text.
fn check_params_for_types(params: &[NodeId], ctx: &mut LintContext<'_>) {
    // Collect param info first to avoid borrow conflicts
    let param_info: Vec<(starlint_ast::types::Span, bool)> = params
        .iter()
        .filter_map(|&param_id| {
            let param_node = ctx.node(param_id)?;
            let param_span = param_node.span();
            let start = usize::try_from(param_span.start).unwrap_or(0);
            let end = usize::try_from(param_span.end).unwrap_or(0);
            let param_text = ctx.source_text().get(start..end).unwrap_or("");
            Some((param_span, param_text.contains(':')))
        })
        .collect();

    for (param_span, has_type) in param_info {
        if !has_type {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Exported function parameter missing explicit type annotation".to_owned(),
                span: Span::new(param_span.start, param_span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ExplicitModuleBoundaryTypes)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_exported_function_missing_return_type() {
        let diags = lint("export function foo() { return 1; }");
        assert!(
            !diags.is_empty(),
            "exported function without return type should be flagged"
        );
    }

    #[test]
    fn test_allows_exported_function_with_return_type() {
        let diags = lint("export function foo(): number { return 1; }");
        assert!(
            diags.is_empty(),
            "exported function with return type should not be flagged"
        );
    }

    #[test]
    fn test_flags_exported_function_missing_param_type() {
        let diags = lint("export function foo(x): number { return x; }");
        assert_eq!(
            diags.len(),
            1,
            "exported function with untyped parameter should be flagged"
        );
    }

    #[test]
    fn test_allows_non_exported_function() {
        let diags = lint("function foo() { return 1; }");
        assert!(
            diags.is_empty(),
            "non-exported function should not be flagged"
        );
    }

    #[test]
    fn test_flags_default_exported_function_missing_return_type() {
        let diags = lint("export default function foo() { return 1; }");
        assert!(
            !diags.is_empty(),
            "default-exported function without return type should be flagged"
        );
    }
}
