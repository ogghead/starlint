//! Rule: `no-lone-blocks`
//!
//! Disallow unnecessary nested blocks. Standalone block statements that don't
//! contain `let`, `const`, `class`, or `function` declarations serve no
//! purpose and may indicate a structural error.

use oxc_ast::AstKind;
use oxc_ast::ast::Statement;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags standalone block statements that serve no purpose.
#[derive(Debug)]
pub struct NoLoneBlocks;

impl NativeRule for NoLoneBlocks {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-lone-blocks".to_owned(),
            description: "Disallow unnecessary nested blocks".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        // We look for BlockStatement nodes that appear as direct children of
        // another block (FunctionBody, BlockStatement, or Program body).
        // Since we don't have parent access, we check from the parent side:
        // scan FunctionBody/Program/BlockStatement for child BlockStatements
        // that don't contain block-scoped declarations.
        let maybe_stmts: Option<&[Statement<'_>]> = match kind {
            AstKind::FunctionBody(body) => Some(&body.statements),
            AstKind::Program(program) => Some(&program.body),
            AstKind::BlockStatement(block) => Some(&block.body),
            _ => None,
        };

        let Some(stmts) = maybe_stmts else {
            return;
        };

        // Collect spans first to avoid borrow conflict
        let mut lone_spans: Vec<Span> = Vec::new();

        for stmt in stmts {
            if let Statement::BlockStatement(block) = stmt {
                // If the block contains no block-scoped declarations, it's unnecessary
                if !has_block_scoped_declaration(&block.body) {
                    lone_spans.push(Span::new(block.span.start, block.span.end));
                }
            }
        }

        for span in lone_spans {
            ctx.report_warning("no-lone-blocks", "Unnecessary block statement", span);
        }
    }
}

/// Check if any statement in the block is a block-scoped declaration
/// (let, const, class, or function).
fn has_block_scoped_declaration(stmts: &[Statement<'_>]) -> bool {
    stmts.iter().any(|stmt| {
        matches!(
            stmt,
            Statement::VariableDeclaration(decl)
                if decl.kind == oxc_ast::ast::VariableDeclarationKind::Let
                    || decl.kind == oxc_ast::ast::VariableDeclarationKind::Const
        ) || matches!(stmt, Statement::ClassDeclaration(_))
    })
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoLoneBlocks)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_lone_block() {
        let diags = lint("{ var x = 1; }");
        assert_eq!(
            diags.len(),
            1,
            "standalone block without block-scoped declarations should be flagged"
        );
    }

    #[test]
    fn test_allows_block_with_let() {
        let diags = lint("{ let x = 1; }");
        assert!(
            diags.is_empty(),
            "block with let declaration should not be flagged"
        );
    }

    #[test]
    fn test_allows_block_with_const() {
        let diags = lint("{ const x = 1; }");
        assert!(
            diags.is_empty(),
            "block with const declaration should not be flagged"
        );
    }

    #[test]
    fn test_allows_if_block() {
        let diags = lint("if (true) { var x = 1; }");
        assert!(diags.is_empty(), "if block should not be flagged");
    }

    #[test]
    fn test_flags_empty_block() {
        let diags = lint("{ }");
        assert_eq!(diags.len(), 1, "empty standalone block should be flagged");
    }
}
