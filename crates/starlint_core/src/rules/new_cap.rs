//! Rule: `new-cap`
//!
//! Require constructor names to begin with a capital letter. Calling `new` on
//! a lowercase identifier is almost always a mistake — constructors should
//! follow the `PascalCase` convention.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `new` expressions where the callee starts with a lowercase letter.
#[derive(Debug)]
pub struct NewCap;

impl NativeRule for NewCap {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "new-cap".to_owned(),
            description: "Require constructor names to begin with a capital letter".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::NewExpression(new_expr) = kind else {
            return;
        };

        // Determine the relevant name to check:
        // - `new foo()` → check "foo"
        // - `new foo.Bar()` → check "Bar" (the last property)
        // - `new foo.bar()` → check "bar"
        let name = match &new_expr.callee {
            Expression::Identifier(ident) => Some(ident.name.as_str()),
            Expression::StaticMemberExpression(member) => Some(member.property.name.as_str()),
            _ => None,
        };

        let Some(callee_name) = name else {
            return;
        };

        // Check if the first character is lowercase
        let first_char = callee_name.chars().next();
        let Some(ch) = first_char else {
            return;
        };

        if ch.is_lowercase() {
            ctx.report_warning(
                "new-cap",
                &format!(
                    "A constructor name `{callee_name}` should start with an uppercase letter"
                ),
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NewCap)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_lowercase_constructor() {
        let diags = lint("var x = new foo();");
        assert_eq!(diags.len(), 1, "new foo() with lowercase should be flagged");
    }

    #[test]
    fn test_allows_uppercase_constructor() {
        let diags = lint("var x = new Foo();");
        assert!(diags.is_empty(), "new Foo() should not be flagged");
    }

    #[test]
    fn test_allows_member_expression_uppercase() {
        let diags = lint("var x = new bar.Baz();");
        assert!(
            diags.is_empty(),
            "new bar.Baz() should not be flagged (checks last property)"
        );
    }

    #[test]
    fn test_flags_member_expression_lowercase() {
        let diags = lint("var x = new bar.baz();");
        assert_eq!(
            diags.len(),
            1,
            "new bar.baz() with lowercase property should be flagged"
        );
    }

    #[test]
    fn test_allows_date_constructor() {
        let diags = lint("var d = new Date();");
        assert!(diags.is_empty(), "new Date() should not be flagged");
    }

    #[test]
    fn test_allows_regular_function_call() {
        let diags = lint("foo();");
        assert!(
            diags.is_empty(),
            "regular function call should not be flagged"
        );
    }

    #[test]
    fn test_allows_uppercase_function_call() {
        let diags = lint("Foo();");
        assert!(
            diags.is_empty(),
            "uppercase function call without new should not be flagged"
        );
    }
}
