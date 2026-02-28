//! Rule: `no-new`
//!
//! Disallow `new` operators with side effects outside of assignments.
//! Using `new` for side effects (e.g. `new Person()`) without assigning
//! the result is wasteful and confusing.

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `new` expressions used as statements (result not stored).
#[derive(Debug)]
pub struct NoNew;

impl NativeRule for NoNew {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-new".to_owned(),
            description: "Disallow `new` for side effects".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        // Look for ExpressionStatement containing a NewExpression
        let AstKind::ExpressionStatement(stmt) = kind else {
            return;
        };

        if matches!(stmt.expression, oxc_ast::ast::Expression::NewExpression(_)) {
            ctx.report_warning(
                "no-new",
                "Do not use `new` for side effects",
                Span::new(stmt.span.start, stmt.span.end),
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNew)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_new_as_statement() {
        let diags = lint("new Person();");
        assert_eq!(diags.len(), 1, "new as statement should be flagged");
    }

    #[test]
    fn test_allows_new_assigned() {
        let diags = lint("var p = new Person();");
        assert!(
            diags.is_empty(),
            "new assigned to variable should not be flagged"
        );
    }

    #[test]
    fn test_allows_function_call_statement() {
        let diags = lint("doSomething();");
        assert!(
            diags.is_empty(),
            "normal function call should not be flagged"
        );
    }
}
