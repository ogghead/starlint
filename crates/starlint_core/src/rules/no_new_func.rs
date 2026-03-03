//! Rule: `no-new-func`
//!
//! Disallow `new Function()`. The `Function` constructor creates functions
//! from strings, similar to `eval()`, and carries the same security risks.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `new Function()` and `Function()` constructor calls.
#[derive(Debug)]
pub struct NoNewFunc;

impl NativeRule for NoNewFunc {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-new-func".to_owned(),
            description: "Disallow `new Function()`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression, AstType::NewExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::NewExpression(new_expr) => {
                if matches!(&new_expr.callee, Expression::Identifier(id) if id.name.as_str() == "Function")
                {
                    ctx.report_warning(
                        "no-new-func",
                        "The `Function` constructor is `eval`",
                        Span::new(new_expr.span.start, new_expr.span.end),
                    );
                }
            }
            AstKind::CallExpression(call) => {
                if matches!(&call.callee, Expression::Identifier(id) if id.name.as_str() == "Function")
                {
                    ctx.report_warning(
                        "no-new-func",
                        "The `Function` constructor is `eval`",
                        Span::new(call.span.start, call.span.end),
                    );
                }
            }
            _ => {}
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNewFunc)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_new_function() {
        let diags = lint("var f = new Function('a', 'return a');");
        assert_eq!(diags.len(), 1, "new Function() should be flagged");
    }

    #[test]
    fn test_flags_function_call() {
        let diags = lint("var f = Function('a', 'return a');");
        assert_eq!(diags.len(), 1, "Function() call should be flagged");
    }

    #[test]
    fn test_allows_normal_constructor() {
        let diags = lint("var x = new MyClass();");
        assert!(diags.is_empty(), "normal constructor should not be flagged");
    }
}
