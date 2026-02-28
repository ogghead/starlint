//! Rule: `prefer-array-some` (unicorn)
//!
//! Prefer `.some()` over `.find()` when only checking for existence.
//! Using `.some()` returns a boolean directly and is more semantically
//! correct when you don't need the found element.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.find()` used in boolean contexts.
#[derive(Debug)]
pub struct PreferArraySome;

impl NativeRule for PreferArraySome {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-array-some".to_owned(),
            description: "Prefer .some() over .find() for existence checks".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        // Look for `if (arr.find(...))` — find used in boolean context
        let AstKind::IfStatement(if_stmt) = kind else {
            return;
        };

        if is_find_call(&if_stmt.test) {
            let span = if_stmt.test.span();
            ctx.report_warning(
                "prefer-array-some",
                "Prefer `.some()` over `.find()` when checking for existence",
                Span::new(span.start, span.end),
            );
        }
    }
}

/// Check if an expression is a `.find(...)` call.
fn is_find_call(expr: &Expression<'_>) -> bool {
    let Expression::CallExpression(call) = expr else {
        return false;
    };

    let Expression::StaticMemberExpression(member) = &call.callee else {
        return false;
    };

    member.property.name == "find"
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferArraySome)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_find_in_if() {
        let diags = lint("if (arr.find(x => x > 0)) { }");
        assert_eq!(diags.len(), 1, "find in if condition should be flagged");
    }

    #[test]
    fn test_allows_some() {
        let diags = lint("if (arr.some(x => x > 0)) { }");
        assert!(diags.is_empty(), "some should not be flagged");
    }

    #[test]
    fn test_allows_find_in_assignment() {
        let diags = lint("var item = arr.find(x => x > 0);");
        assert!(diags.is_empty(), "find in assignment should not be flagged");
    }
}
