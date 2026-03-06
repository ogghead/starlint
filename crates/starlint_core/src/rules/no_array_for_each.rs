//! Rule: `no-array-for-each`
//!
//! Disallow `Array#forEach()`. Prefer `for...of` loops for iterating
//! over arrays. `for...of` supports `break`, `continue`, and `await`,
//! and avoids the overhead of a callback function.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags calls to `.forEach()`.
#[derive(Debug)]
pub struct NoArrayForEach;

impl NativeRule for NoArrayForEach {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-array-for-each".to_owned(),
            description: "Disallow Array#forEach()".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "forEach" {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "no-array-for-each".to_owned(),
            message: "Prefer `for...of` over `.forEach()`".to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoArrayForEach)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_for_each_arrow() {
        let diags = lint("arr.forEach(x => console.log(x));");
        assert_eq!(diags.len(), 1, ".forEach() with arrow should be flagged");
    }

    #[test]
    fn test_flags_for_each_function() {
        let diags = lint("arr.forEach(function(x) { console.log(x); });");
        assert_eq!(
            diags.len(),
            1,
            ".forEach() with function expression should be flagged"
        );
    }

    #[test]
    fn test_allows_for_of() {
        let diags = lint("for (const x of arr) { console.log(x); }");
        assert!(diags.is_empty(), "for...of should not be flagged");
    }

    #[test]
    fn test_allows_map() {
        let diags = lint("arr.map(x => x + 1);");
        assert!(diags.is_empty(), ".map() should not be flagged");
    }

    #[test]
    fn test_allows_filter() {
        let diags = lint("arr.filter(x => x > 0);");
        assert!(diags.is_empty(), ".filter() should not be flagged");
    }

    #[test]
    fn test_allows_reduce() {
        let diags = lint("arr.reduce((acc, x) => acc + x, 0);");
        assert!(diags.is_empty(), ".reduce() should not be flagged");
    }
}
