//! Rule: `no-void`
//!
//! Disallow the `void` operator. The `void` operator is rarely needed
//! and can be confusing.

use oxc_ast::AstKind;
use oxc_ast::ast::UnaryOperator;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags use of the `void` operator.
#[derive(Debug)]
pub struct NoVoid;

impl NativeRule for NoVoid {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-void".to_owned(),
            description: "Disallow the `void` operator".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::UnaryExpression(unary) = kind else {
            return;
        };

        if unary.operator == UnaryOperator::Void {
            ctx.report_warning(
                "no-void",
                "Expected `undefined` instead of `void`",
                Span::new(unary.span.start, unary.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoVoid)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_void_operator() {
        let diags = lint("var x = void 0;");
        assert_eq!(diags.len(), 1, "void operator should be flagged");
    }

    #[test]
    fn test_allows_undefined() {
        let diags = lint("var x = undefined;");
        assert!(diags.is_empty(), "undefined should not be flagged");
    }
}
