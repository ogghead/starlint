//! Rule: `node/no-path-concat`
//!
//! Disallow string concatenation with `__dirname` or `__filename`.
//! Building file paths with `+` is fragile and platform-dependent.
//! Use `path.join()` or `path.resolve()` instead.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags string concatenation (`+`) involving `__dirname` or `__filename`.
#[derive(Debug)]
pub struct NoPathConcat;

/// Check whether an expression is an identifier named `__dirname` or `__filename`.
fn is_path_global(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::Identifier(id)
            if id.name.as_str() == "__dirname" || id.name.as_str() == "__filename"
    )
}

impl NativeRule for NoPathConcat {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "node/no-path-concat".to_owned(),
            description: "Disallow string concatenation with `__dirname` or `__filename`"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::BinaryExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::BinaryExpression(expr) = kind else {
            return;
        };

        if expr.operator != BinaryOperator::Addition {
            return;
        }

        if !is_path_global(&expr.left) && !is_path_global(&expr.right) {
            return;
        }

        ctx.report_warning(
            "node/no-path-concat",
            "Do not concatenate paths with `+` \u{2014} use `path.join()` or `path.resolve()` instead",
            Span::new(expr.span.start, expr.span.end),
        );
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoPathConcat)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_dirname_concat() {
        let diags = lint("var p = __dirname + '/foo';");
        assert_eq!(diags.len(), 1, "__dirname + string should be flagged");
    }

    #[test]
    fn test_flags_filename_concat() {
        let diags = lint("var p = '/bar' + __filename;");
        assert_eq!(diags.len(), 1, "string + __filename should be flagged");
    }

    #[test]
    fn test_allows_path_join() {
        let diags = lint("var p = path.join(__dirname, 'foo');");
        assert!(diags.is_empty(), "path.join should not be flagged");
    }

    #[test]
    fn test_allows_normal_concat() {
        let diags = lint("var s = a + b;");
        assert!(
            diags.is_empty(),
            "normal concatenation should not be flagged"
        );
    }

    #[test]
    fn test_allows_string_addition() {
        let diags = lint("var s = 'hello' + 'world';");
        assert!(
            diags.is_empty(),
            "normal string addition should not be flagged"
        );
    }
}
