//! Rule: `init-declarations`
//!
//! Require initialization in variable declarations (default mode: "always").
//! Variables declared without an initializer are a potential source of
//! `undefined`-related bugs. However, for-in and for-of loop variables are
//! assigned implicitly and are exempt.
//!
//! Uses semantic analysis to check whether a `VariableDeclaration` is the
//! left-hand side of a `for-in` or `for-of` statement.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags variable declarations without initializers.
#[derive(Debug)]
pub struct InitDeclarations;

/// Check whether a variable declaration is the `left` of a for-in or for-of.
///
/// Walks the semantic ancestor chain to find the immediate parent statement.
fn is_for_in_or_for_of_left(
    node_id: oxc_semantic::NodeId,
    semantic: &oxc_semantic::Semantic<'_>,
) -> bool {
    let nodes = semantic.nodes();
    for ancestor in nodes.ancestors(node_id) {
        match ancestor.kind() {
            AstKind::ForInStatement(_) | AstKind::ForOfStatement(_) => return true,
            // Stop at statements or declarations that cannot be parents of a for-left.
            AstKind::Program(_)
            | AstKind::Function(_)
            | AstKind::ArrowFunctionExpression(_)
            | AstKind::ExpressionStatement(_)
            | AstKind::BlockStatement(_)
            | AstKind::IfStatement(_)
            | AstKind::WhileStatement(_)
            | AstKind::DoWhileStatement(_)
            | AstKind::ForStatement(_)
            | AstKind::SwitchStatement(_) => return false,
            _ => {}
        }
    }
    false
}

/// Find the semantic [`NodeId`] for a node with the given span.
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

impl NativeRule for InitDeclarations {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "init-declarations".to_owned(),
            description: "Require initialization in variable declarations".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn needs_semantic(&self) -> bool {
        true
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[
            AstType::ArrowFunctionExpression,
            AstType::BlockStatement,
            AstType::DoWhileStatement,
            AstType::ExpressionStatement,
            AstType::ForInStatement,
            AstType::ForOfStatement,
            AstType::ForStatement,
            AstType::Function,
            AstType::IfStatement,
            AstType::Program,
            AstType::SwitchStatement,
            AstType::VariableDeclaration,
            AstType::WhileStatement,
        ])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::VariableDeclaration(decl) = kind else {
            return;
        };

        let Some(semantic) = ctx.semantic() else {
            return;
        };

        // Check if this declaration is the left-hand side of a for-in/for-of.
        if let Some(node_id) = find_node_id_by_span(semantic, decl.span) {
            if is_for_in_or_for_of_left(node_id, semantic) {
                return;
            }
        }

        // Check each declarator for missing initializers.
        for declarator in &decl.declarations {
            if declarator.init.is_none() {
                let span = declarator.span();
                ctx.report_warning(
                    "init-declarations",
                    "Variable declaration should be initialized",
                    Span::new(span.start, span.end),
                );
            }
        }
    }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(InitDeclarations)];
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
    fn test_flags_var_without_init() {
        let diags = lint("var x;");
        assert_eq!(diags.len(), 1, "var without init should be flagged");
    }

    #[test]
    fn test_allows_var_with_init() {
        let diags = lint("var x = 1;");
        assert!(diags.is_empty(), "var with init should not be flagged");
    }

    #[test]
    fn test_flags_let_without_init() {
        let diags = lint("let x;");
        assert_eq!(diags.len(), 1, "let without init should be flagged");
    }

    #[test]
    fn test_allows_let_with_init() {
        let diags = lint("let x = 1;");
        assert!(diags.is_empty(), "let with init should not be flagged");
    }

    #[test]
    fn test_allows_const_with_init() {
        let diags = lint("const x = 1;");
        assert!(
            diags.is_empty(),
            "const always has init and should not be flagged"
        );
    }

    #[test]
    fn test_allows_for_in_var() {
        let diags = lint("for (var x in obj) {}");
        assert!(
            diags.is_empty(),
            "for-in variable should not be flagged (implicitly assigned)"
        );
    }

    #[test]
    fn test_allows_for_of_let() {
        let diags = lint("for (let x of arr) {}");
        assert!(
            diags.is_empty(),
            "for-of variable should not be flagged (implicitly assigned)"
        );
    }

    #[test]
    fn test_flags_multiple_uninit_declarators() {
        let diags = lint("var a, b;");
        assert_eq!(
            diags.len(),
            2,
            "two uninitialised declarators should produce two diagnostics"
        );
    }

    #[test]
    fn test_flags_only_uninit_declarator() {
        let diags = lint("var a = 1, b;");
        assert_eq!(
            diags.len(),
            1,
            "only the uninitialized declarator should be flagged"
        );
    }

    #[test]
    fn test_allows_for_of_destructuring() {
        let diags = lint("for (const [a, b] of pairs) {}");
        assert!(
            diags.is_empty(),
            "for-of destructuring should not be flagged"
        );
    }
}
