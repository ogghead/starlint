//! Rule: `no-useless-collection-argument`
//!
//! Flag unnecessary empty array arguments passed to `new Set()`, `new Map()`,
//! `new WeakSet()`, or `new WeakMap()`. Passing `[]` is equivalent to calling
//! the constructor with no arguments.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `new Set([])`, `new Map([])`, `new WeakSet([])`, and `new WeakMap([])`.
#[derive(Debug)]
pub struct NoUselessCollectionArgument;

/// Collection constructor names that accept an iterable.
const COLLECTION_TYPES: &[&str] = &["Set", "Map", "WeakSet", "WeakMap"];

impl NativeRule for NoUselessCollectionArgument {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-collection-argument".to_owned(),
            description: "Disallow passing an empty array to collection constructors".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::NewExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::NewExpression(new_expr) = kind else {
            return;
        };

        let Expression::Identifier(id) = &new_expr.callee else {
            return;
        };

        let name = id.name.as_str();
        if !COLLECTION_TYPES.contains(&name) {
            return;
        }

        // Check if first argument is an empty array literal `[]`
        let Some(first_arg) = new_expr.arguments.first() else {
            return;
        };

        if is_empty_array_argument(first_arg) {
            let expr_span = Span::new(new_expr.span.start, new_expr.span.end);
            // Remove the empty array argument
            let arg_span = Span::new(first_arg.span().start, first_arg.span().end);
            ctx.report(Diagnostic {
                rule_name: "no-useless-collection-argument".to_owned(),
                message: format!("Unnecessary empty array argument in `new {name}([])` — use `new {name}()` instead"),
                span: expr_span,
                severity: Severity::Warning,
                help: Some(format!("Use `new {name}()` instead")),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Remove empty array argument".to_owned(),
                    edits: vec![Edit {
                        span: arg_span,
                        replacement: String::new(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Check if an argument is an empty array expression `[]`.
fn is_empty_array_argument(arg: &Argument<'_>) -> bool {
    matches!(arg, Argument::ArrayExpression(arr) if arr.elements.is_empty())
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUselessCollectionArgument)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_new_set_empty_array() {
        let diags = lint("new Set([]);");
        assert_eq!(diags.len(), 1, "new Set([]) should be flagged");
    }

    #[test]
    fn test_flags_new_map_empty_array() {
        let diags = lint("new Map([]);");
        assert_eq!(diags.len(), 1, "new Map([]) should be flagged");
    }

    #[test]
    fn test_flags_new_weakset_empty_array() {
        let diags = lint("new WeakSet([]);");
        assert_eq!(diags.len(), 1, "new WeakSet([]) should be flagged");
    }

    #[test]
    fn test_flags_new_weakmap_empty_array() {
        let diags = lint("new WeakMap([]);");
        assert_eq!(diags.len(), 1, "new WeakMap([]) should be flagged");
    }

    #[test]
    fn test_allows_no_argument() {
        let diags = lint("new Set();");
        assert!(diags.is_empty(), "new Set() should not be flagged");
    }

    #[test]
    fn test_allows_non_empty_array() {
        let diags = lint("new Set([1, 2]);");
        assert!(diags.is_empty(), "new Set([1, 2]) should not be flagged");
    }

    #[test]
    fn test_allows_variable_argument() {
        let diags = lint("new Set(items);");
        assert!(diags.is_empty(), "new Set(items) should not be flagged");
    }

    #[test]
    fn test_allows_non_collection_constructor() {
        let diags = lint("new Array([]);");
        assert!(
            diags.is_empty(),
            "new Array([]) should not be flagged by this rule"
        );
    }
}
