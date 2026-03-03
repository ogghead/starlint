//! Rule: `no-useless-undefined` (unicorn)
//!
//! Disallow useless `undefined`. Using `undefined` as a default value,
//! return value, or argument is usually unnecessary since JavaScript
//! provides it implicitly.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags useless uses of `undefined`.
#[derive(Debug)]
pub struct NoUselessUndefined;

impl NativeRule for NoUselessUndefined {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-undefined".to_owned(),
            description: "Disallow useless undefined".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ReturnStatement, AstType::VariableDeclarator])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            // `let x = undefined;` or `const x = undefined;`
            AstKind::VariableDeclarator(decl) => {
                if let Some(init) = &decl.init {
                    if is_undefined(init) {
                        ctx.report_warning(
                            "no-useless-undefined",
                            "Do not use useless `undefined`",
                            Span::new(decl.span.start, decl.span.end),
                        );
                    }
                }
            }
            // `return undefined;`
            AstKind::ReturnStatement(ret) => {
                if let Some(arg) = &ret.argument {
                    if is_undefined(arg) {
                        ctx.report_warning(
                            "no-useless-undefined",
                            "Do not use useless `undefined`",
                            Span::new(ret.span.start, ret.span.end),
                        );
                    }
                }
            }
            // `void 0` is a different pattern (intentional), skip it
            _ => {}
        }
    }
}

/// Check if an expression is `undefined` (the identifier).
fn is_undefined(expr: &Expression<'_>) -> bool {
    matches!(expr, Expression::Identifier(ident) if ident.name == "undefined")
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUselessUndefined)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_let_undefined() {
        let diags = lint("let x = undefined;");
        assert_eq!(diags.len(), 1, "let x = undefined should be flagged");
    }

    #[test]
    fn test_flags_return_undefined() {
        let diags = lint("function foo() { return undefined; }");
        assert_eq!(diags.len(), 1, "return undefined should be flagged");
    }

    #[test]
    fn test_allows_let_with_value() {
        let diags = lint("let x = 1;");
        assert!(diags.is_empty(), "let with value should not be flagged");
    }

    #[test]
    fn test_allows_return_nothing() {
        let diags = lint("function foo() { return; }");
        assert!(diags.is_empty(), "bare return should not be flagged");
    }

    #[test]
    fn test_allows_let_no_init() {
        let diags = lint("let x;");
        assert!(diags.is_empty(), "let without init should not be flagged");
    }
}
