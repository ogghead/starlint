//! Rule: `no-extra-bind`
//!
//! Disallow unnecessary `.bind()` calls. If a function does not use `this`,
//! calling `.bind()` on it is unnecessary.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.bind()` calls on arrow functions (which cannot be rebound).
#[derive(Debug)]
pub struct NoExtraBind;

impl NativeRule for NoExtraBind {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-extra-bind".to_owned(),
            description: "Disallow unnecessary `.bind()` calls".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check for `.bind()` call
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "bind" {
            return;
        }

        // Arrow functions cannot be rebound — `.bind()` on them is always useless
        if is_arrow_function(&member.object) {
            ctx.report_warning(
                "no-extra-bind",
                "The `.bind()` call on an arrow function is unnecessary",
                Span::new(call.span.start, call.span.end),
            );
        }
    }
}

/// Check if an expression is an arrow function, unwrapping parenthesized expressions.
fn is_arrow_function(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::ArrowFunctionExpression(_) => true,
        Expression::ParenthesizedExpression(paren) => is_arrow_function(&paren.expression),
        _ => false,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoExtraBind)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_bind_on_arrow() {
        let diags = lint("var f = (() => {}).bind(this);");
        assert_eq!(
            diags.len(),
            1,
            ".bind() on arrow function should be flagged"
        );
    }

    #[test]
    fn test_allows_bind_on_function() {
        let diags = lint("var f = function() { return this; }.bind(obj);");
        assert!(
            diags.is_empty(),
            ".bind() on regular function should not be flagged"
        );
    }

    #[test]
    fn test_allows_normal_call() {
        let diags = lint("var f = foo();");
        assert!(diags.is_empty(), "normal call should not be flagged");
    }
}
