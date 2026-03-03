//! Rule: `arrow-body-style`
//!
//! Enforce consistent arrow function body style. When an arrow function body
//! contains only a single `return` statement, the block body can be replaced
//! with an expression body.

use oxc_ast::AstKind;
use oxc_ast::ast::Statement;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags arrow functions with block bodies that could use expression bodies.
#[derive(Debug)]
pub struct ArrowBodyStyle;

impl NativeRule for ArrowBodyStyle {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "arrow-body-style".to_owned(),
            description: "Enforce consistent arrow function body style".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ArrowFunctionExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ArrowFunctionExpression(arrow) = kind else {
            return;
        };

        // Only check block-body arrows (expression == false means block body)
        if arrow.expression {
            return;
        }

        // Check if body has exactly one statement that is a return with an argument
        if arrow.body.statements.len() != 1 {
            return;
        }

        let Some(stmt) = arrow.body.statements.first() else {
            return;
        };

        if let Statement::ReturnStatement(ret) = stmt {
            if ret.argument.is_some() {
                ctx.report_warning(
                    "arrow-body-style",
                    "Unexpected block statement surrounding arrow body; move the returned value immediately after `=>`",
                    Span::new(arrow.span.start, arrow.span.end),
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
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ArrowBodyStyle)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_block_body_with_single_return() {
        let diags = lint("const f = () => { return 1; };");
        assert_eq!(
            diags.len(),
            1,
            "block body with single return should be flagged"
        );
    }

    #[test]
    fn test_allows_expression_body() {
        let diags = lint("const f = () => 1;");
        assert!(diags.is_empty(), "expression body should not be flagged");
    }

    #[test]
    fn test_allows_multiple_statements() {
        let diags = lint("const f = () => { const x = 1; return x; };");
        assert!(
            diags.is_empty(),
            "multiple statements should not be flagged"
        );
    }

    #[test]
    fn test_allows_empty_return() {
        let diags = lint("const f = () => { return; };");
        assert!(
            diags.is_empty(),
            "return without argument should not be flagged"
        );
    }

    #[test]
    fn test_allows_block_body_no_return() {
        let diags = lint("const f = () => { console.log('hi'); };");
        assert!(
            diags.is_empty(),
            "block body without return should not be flagged"
        );
    }
}
