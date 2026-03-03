//! Rule: `throw-new-error`
//!
//! Require `new` when throwing Error constructors. `throw Error("msg")` works
//! but is inconsistent — `throw new Error("msg")` is the standard form.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Standard JavaScript error constructors.
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

/// Flags `throw Error(...)` expressions that are missing `new`.
#[derive(Debug)]
pub struct ThrowNewError;

impl NativeRule for ThrowNewError {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "throw-new-error".to_owned(),
            description: "Require `new` when throwing Error constructors".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ThrowStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ThrowStatement(stmt) = kind else {
            return;
        };

        // Must be a direct call: `throw Error(...)`, not `throw new Error(...)`.
        let Expression::CallExpression(call) = &stmt.argument else {
            return;
        };

        // Callee must be a simple identifier (not a member expression).
        let Expression::Identifier(id) = &call.callee else {
            return;
        };

        let name = id.name.as_str();
        if !ERROR_CONSTRUCTORS.contains(&name) {
            return;
        }

        // Build fix: insert `new ` before the error constructor name.
        let callee_start = id.span.start;

        ctx.report(Diagnostic {
            rule_name: "throw-new-error".to_owned(),
            message: format!("Use `new {name}()` instead of `{name}()`"),
            span: Span::new(stmt.span.start, stmt.span.end),
            severity: Severity::Error,
            help: Some(format!("Add `new` before `{name}`")),
            fix: Some(Fix {
                message: format!("Add `new` before `{name}`"),
                edits: vec![Edit {
                    span: Span::new(callee_start, callee_start),
                    replacement: "new ".to_owned(),
                }],
            }),
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ThrowNewError)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_throw_error() {
        let diags = lint("throw Error('msg');");
        assert_eq!(diags.len(), 1, "should flag throw Error()");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert!(fix.is_some(), "should provide a fix");
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("new "),
            "fix should insert 'new '"
        );
    }

    #[test]
    fn test_flags_throw_type_error() {
        let diags = lint("throw TypeError('msg');");
        assert_eq!(diags.len(), 1, "should flag throw TypeError()");
    }

    #[test]
    fn test_flags_throw_range_error() {
        let diags = lint("throw RangeError('msg');");
        assert_eq!(diags.len(), 1, "should flag throw RangeError()");
    }

    #[test]
    fn test_allows_throw_new_error() {
        let diags = lint("throw new Error('msg');");
        assert!(diags.is_empty(), "throw new Error() should not be flagged");
    }

    #[test]
    fn test_allows_throw_variable() {
        let diags = lint("throw err;");
        assert!(diags.is_empty(), "throw variable should not be flagged");
    }

    #[test]
    fn test_allows_throw_string() {
        let diags = lint("throw 'error';");
        assert!(
            diags.is_empty(),
            "throw string literal should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_error_call() {
        let diags = lint("throw myFunction('msg');");
        assert!(
            diags.is_empty(),
            "non-error function call should not be flagged"
        );
    }
}
