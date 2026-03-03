//! Rule: `node/no-new-require`
//!
//! Disallow `new require('...')`. The `require` function is not a
//! constructor. Using `new` with it is almost always a mistake \u{2014}
//! typically the intent is `new (require('module'))()`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `new require(...)` expressions.
#[derive(Debug)]
pub struct NoNewRequire;

impl NativeRule for NoNewRequire {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "node/no-new-require".to_owned(),
            description: "Disallow `new require(...)`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::NewExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::NewExpression(new_expr) = kind else {
            return;
        };

        let is_require = matches!(
            &new_expr.callee,
            Expression::Identifier(id) if id.name.as_str() == "require"
        );

        if is_require {
            ctx.report_error(
                "node/no-new-require",
                "`require` is not a constructor \u{2014} use `new (require('module'))()` to instantiate the export",
                Span::new(new_expr.span.start, new_expr.span.end),
            );
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNewRequire)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_new_require() {
        let diags = lint("var x = new require('x');");
        assert_eq!(diags.len(), 1, "new require() should be flagged");
    }

    #[test]
    fn test_allows_plain_require() {
        let diags = lint("var x = require('x');");
        assert!(diags.is_empty(), "plain require() should not be flagged");
    }

    #[test]
    fn test_allows_new_other_constructor() {
        let diags = lint("var x = new Foo();");
        assert!(diags.is_empty(), "new Foo() should not be flagged");
    }

    #[test]
    fn test_flags_new_require_with_path() {
        let diags = lint("var app = new require('./app');");
        assert_eq!(diags.len(), 1, "new require with path should be flagged");
    }
}
