//! Rule: `missing-throw` (OXC)
//!
//! Detect `new Error()` (or subclasses) used as an expression statement without
//! `throw`. Creating an error without throwing it is almost always a mistake.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Known error constructor names.
const ERROR_CONSTRUCTORS: &[&str] = &[
    "Error",
    "TypeError",
    "RangeError",
    "ReferenceError",
    "SyntaxError",
    "URIError",
    "EvalError",
    "AggregateError",
];

/// Flags `new Error()` without `throw`.
#[derive(Debug)]
pub struct MissingThrow;

impl NativeRule for MissingThrow {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "missing-throw".to_owned(),
            description: "Detect `new Error()` without `throw`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ExpressionStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        // We look for ExpressionStatement where the expression is a NewExpression
        // with an error constructor callee.
        let AstKind::ExpressionStatement(stmt) = kind else {
            return;
        };

        let Expression::NewExpression(new_expr) = &stmt.expression else {
            return;
        };

        let is_error_ctor = match &new_expr.callee {
            Expression::Identifier(id) => ERROR_CONSTRUCTORS.contains(&id.name.as_str()),
            _ => false,
        };

        if is_error_ctor {
            ctx.report_warning(
                "missing-throw",
                "`new Error()` is not thrown — did you forget `throw`?",
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(MissingThrow)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_new_error_without_throw() {
        let diags = lint("new Error('oops');");
        assert_eq!(diags.len(), 1, "new Error without throw should be flagged");
    }

    #[test]
    fn test_flags_new_type_error_without_throw() {
        let diags = lint("new TypeError('bad');");
        assert_eq!(
            diags.len(),
            1,
            "new TypeError without throw should be flagged"
        );
    }

    #[test]
    fn test_allows_throw_new_error() {
        let diags = lint("throw new Error('oops');");
        assert!(diags.is_empty(), "throw new Error should not be flagged");
    }

    #[test]
    fn test_allows_assigned_error() {
        let diags = lint("const e = new Error('oops');");
        assert!(diags.is_empty(), "assigned new Error should not be flagged");
    }

    #[test]
    fn test_allows_non_error_constructor() {
        let diags = lint("new Map();");
        assert!(
            diags.is_empty(),
            "non-error constructor should not be flagged"
        );
    }
}
