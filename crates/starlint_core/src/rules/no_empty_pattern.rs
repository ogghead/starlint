//! Rule: `no-empty-pattern`
//!
//! Disallow empty destructuring patterns. An empty pattern like `const {} = foo`
//! or `const [] = bar` looks like a destructuring assignment but doesn't
//! actually create any bindings. It almost always indicates a typo where the
//! developer meant to use a default value `{ a = {} }` instead of destructuring
//! `{ a: {} }`.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags empty destructuring patterns (empty object `{}` or array `[]` patterns).
#[derive(Debug)]
pub struct NoEmptyPattern;

impl NativeRule for NoEmptyPattern {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-empty-pattern".to_owned(),
            description: "Disallow empty destructuring patterns".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ArrayPattern, AstType::ObjectPattern])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::ObjectPattern(pat) if pat.properties.is_empty() && pat.rest.is_none() => {
                ctx.report_error(
                    "no-empty-pattern",
                    "Unexpected empty object pattern",
                    Span::new(pat.span.start, pat.span.end),
                );
            }
            AstKind::ArrayPattern(pat) if pat.elements.is_empty() && pat.rest.is_none() => {
                ctx.report_error(
                    "no-empty-pattern",
                    "Unexpected empty array pattern",
                    Span::new(pat.span.start, pat.span.end),
                );
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoEmptyPattern)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_empty_object_pattern() {
        let diags = lint("const {} = foo;");
        assert_eq!(diags.len(), 1, "empty object pattern should be flagged");
    }

    #[test]
    fn test_flags_empty_array_pattern() {
        let diags = lint("const [] = foo;");
        assert_eq!(diags.len(), 1, "empty array pattern should be flagged");
    }

    #[test]
    fn test_flags_empty_pattern_in_params() {
        let diags = lint("function f({}) {}");
        assert_eq!(diags.len(), 1, "empty pattern in params should be flagged");
    }

    #[test]
    fn test_allows_non_empty_object_pattern() {
        let diags = lint("const { a } = foo;");
        assert!(
            diags.is_empty(),
            "non-empty object pattern should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_empty_array_pattern() {
        let diags = lint("const [a] = foo;");
        assert!(
            diags.is_empty(),
            "non-empty array pattern should not be flagged"
        );
    }

    #[test]
    fn test_allows_rest_element() {
        let diags = lint("const [...rest] = foo;");
        assert!(
            diags.is_empty(),
            "rest element in array pattern should not be flagged"
        );
    }

    #[test]
    fn test_allows_rest_in_object() {
        let diags = lint("const { ...rest } = foo;");
        assert!(
            diags.is_empty(),
            "rest in object pattern should not be flagged"
        );
    }
}
