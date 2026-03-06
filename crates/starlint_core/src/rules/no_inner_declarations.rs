//! Rule: `no-inner-declarations`
//!
//! Disallow variable or function declarations in nested blocks.
//! Prior to ES6, function declarations were only allowed in the program body
//! or a function body. While ES6 allows block-level functions in strict mode,
//! `var` declarations in blocks are still hoisted and can be confusing.
//!
//! Uses semantic analysis to walk the ancestor chain and determine whether a
//! declaration is inside a nested block (not directly in a program or function body).

use oxc_ast::AstKind;
use oxc_ast::ast::VariableDeclarationKind;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags function or `var` declarations inside nested blocks.
#[derive(Debug)]
pub struct NoInnerDeclarations;

/// Check whether the given node is directly inside a valid (non-nested) context.
///
/// A declaration is valid if its nearest enclosing statement container is:
/// - The program body
/// - A function body (function, arrow, method, etc.)
/// - A static block
///
/// It is invalid (inner) if the nearest container is a block inside an
/// `if`/`for`/`while`/`switch`/etc.
fn is_in_valid_position(
    node_id: oxc_semantic::NodeId,
    semantic: &oxc_semantic::Semantic<'_>,
) -> bool {
    let nodes = semantic.nodes();

    for ancestor in nodes.ancestors(node_id) {
        match ancestor.kind() {
            // Program, function body, or static block → valid position.
            AstKind::Program(_)
            | AstKind::Function(_)
            | AstKind::ArrowFunctionExpression(_)
            | AstKind::StaticBlock(_) => return true,

            // Control-flow / nesting constructs → invalid (inner) position.
            AstKind::IfStatement(_)
            | AstKind::ForStatement(_)
            | AstKind::ForInStatement(_)
            | AstKind::ForOfStatement(_)
            | AstKind::WhileStatement(_)
            | AstKind::DoWhileStatement(_)
            | AstKind::SwitchStatement(_)
            | AstKind::SwitchCase(_)
            | AstKind::WithStatement(_)
            | AstKind::TryStatement(_)
            | AstKind::CatchClause(_) => return false,

            // Everything else (block, export, label, function body, exprs) —
            // keep walking up the ancestor chain.
            _ => {}
        }
    }

    // If we exhaust ancestors without finding a container, treat as valid.
    true
}

impl NativeRule for NoInnerDeclarations {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-inner-declarations".to_owned(),
            description: "Disallow variable or function declarations in nested blocks".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn needs_semantic(&self) -> bool {
        true
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[
            AstType::ArrowFunctionExpression,
            AstType::CatchClause,
            AstType::DoWhileStatement,
            AstType::ForInStatement,
            AstType::ForOfStatement,
            AstType::ForStatement,
            AstType::Function,
            AstType::IfStatement,
            AstType::Program,
            AstType::StaticBlock,
            AstType::SwitchCase,
            AstType::SwitchStatement,
            AstType::TryStatement,
            AstType::VariableDeclaration,
            AstType::WhileStatement,
            AstType::WithStatement,
        ])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let Some(semantic) = ctx.semantic() else {
            return;
        };

        match kind {
            AstKind::Function(func) => {
                // Only flag function declarations (not expressions or arrow functions).
                if !func.is_declaration() {
                    return;
                }

                // Find this function's node ID in the semantic tree.
                let Some(node_id) = find_node_id_by_span(semantic, func.span) else {
                    return;
                };

                if !is_in_valid_position(node_id, semantic) {
                    let name = func.id.as_ref().map_or("anonymous", |id| id.name.as_str());
                    ctx.report(Diagnostic {
                        rule_name: "no-inner-declarations".to_owned(),
                        message: format!(
                            "Move function declaration '{name}' to program or function body root"
                        ),
                        span: starlint_plugin_sdk::diagnostic::Span::new(
                            func.span.start,
                            func.span.end,
                        ),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            AstKind::VariableDeclaration(decl) => {
                // Only flag `var` declarations (let/const are block-scoped by design).
                if decl.kind != VariableDeclarationKind::Var {
                    return;
                }

                let Some(node_id) = find_node_id_by_span(semantic, decl.span) else {
                    return;
                };

                if !is_in_valid_position(node_id, semantic) {
                    ctx.report(Diagnostic {
                        rule_name: "no-inner-declarations".to_owned(),
                        message: "Move variable declaration to program or function body root"
                            .to_owned(),
                        span: starlint_plugin_sdk::diagnostic::Span::new(
                            decl.span.start,
                            decl.span.end,
                        ),
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

/// Find the semantic [`NodeId`] for a node with the given span.
///
/// Iterates semantic nodes to find a match. Returns `None` if no node matches.
fn find_node_id_by_span(
    semantic: &oxc_semantic::Semantic<'_>,
    span: oxc_span::Span,
) -> Option<oxc_semantic::NodeId> {
    for node in semantic.nodes() {
        if node.kind().span() == span {
            return Some(node.id());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::{build_semantic, parse_file};
    use crate::traversal::traverse_and_lint_with_semantic;

    /// Helper to lint source code with semantic analysis.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let program = allocator.alloc(parsed.program);
            let semantic = build_semantic(program);
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoInnerDeclarations)];
            traverse_and_lint_with_semantic(
                program,
                &rules,
                source,
                Path::new("test.js"),
                Some(&semantic),
            )
        } else {
            vec![]
        }
    }

    #[test]
    fn test_allows_top_level_function() {
        let diags = lint("function foo() {}");
        assert!(diags.is_empty(), "top-level function should not be flagged");
    }

    #[test]
    fn test_allows_top_level_var() {
        let diags = lint("var x = 1;");
        assert!(diags.is_empty(), "top-level var should not be flagged");
    }

    #[test]
    fn test_allows_let_in_block() {
        let diags = lint("if (true) { let x = 1; }");
        assert!(diags.is_empty(), "let in block should not be flagged");
    }

    #[test]
    fn test_allows_const_in_block() {
        let diags = lint("if (true) { const x = 1; }");
        assert!(diags.is_empty(), "const in block should not be flagged");
    }

    #[test]
    fn test_flags_function_in_if() {
        let diags = lint("if (true) { function foo() {} }");
        assert_eq!(diags.len(), 1, "function in if-block should be flagged");
        assert!(
            diags
                .first()
                .is_some_and(|d| d.rule_name == "no-inner-declarations"),
            "should come from no-inner-declarations"
        );
    }

    #[test]
    fn test_flags_var_in_if() {
        let diags = lint("if (true) { var x = 1; }");
        assert_eq!(diags.len(), 1, "var in if-block should be flagged");
    }

    #[test]
    fn test_flags_function_in_for() {
        let diags = lint("for (;;) { function foo() {} }");
        assert_eq!(diags.len(), 1, "function in for-loop should be flagged");
    }

    #[test]
    fn test_flags_var_in_while() {
        let diags = lint("while (true) { var x = 1; }");
        assert_eq!(diags.len(), 1, "var in while-loop should be flagged");
    }

    #[test]
    fn test_allows_function_in_function_body() {
        let diags = lint("function outer() { function inner() {} }");
        assert!(
            diags.is_empty(),
            "function in function body should not be flagged"
        );
    }

    #[test]
    fn test_allows_var_in_function_body() {
        let diags = lint("function outer() { var x = 1; }");
        assert!(
            diags.is_empty(),
            "var in function body should not be flagged"
        );
    }

    #[test]
    fn test_flags_function_in_nested_if_inside_function() {
        let diags = lint("function outer() { if (true) { function inner() {} } }");
        assert_eq!(
            diags.len(),
            1,
            "function in nested if inside function should be flagged"
        );
    }

    #[test]
    fn test_flags_var_in_try() {
        let diags = lint("try { var x = 1; } catch(e) {}");
        assert_eq!(diags.len(), 1, "var in try block should be flagged");
    }
}
