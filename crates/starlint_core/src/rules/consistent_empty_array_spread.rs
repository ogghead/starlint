//! Rule: `consistent-empty-array-spread`
//!
//! Flag spreading an empty array literal (`..[]`) inside an array expression.
//! Patterns like `[...arr, ...[]]` or `[...[]]` are useless — the empty
//! spread contributes no elements.

use oxc_ast::AstKind;
use oxc_ast::ast::{ArrayExpressionElement, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags spreading an empty array literal inside an array expression.
#[derive(Debug)]
pub struct ConsistentEmptyArraySpread;

/// Check whether a spread element's argument is an empty array literal.
fn is_empty_array_spread(element: &ArrayExpressionElement<'_>) -> bool {
    let ArrayExpressionElement::SpreadElement(spread) = element else {
        return false;
    };

    matches!(&spread.argument, Expression::ArrayExpression(arr) if arr.elements.is_empty())
}

impl NativeRule for ConsistentEmptyArraySpread {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "consistent-empty-array-spread".to_owned(),
            description: "Disallow spreading an empty array literal".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ArrayExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ArrayExpression(arr) = kind else {
            return;
        };

        let has_empty_spread = arr.elements.iter().any(is_empty_array_spread);

        if has_empty_spread {
            ctx.report_warning(
                "consistent-empty-array-spread",
                "Spreading an empty array literal is unnecessary",
                Span::new(arr.span.start, arr.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ConsistentEmptyArraySpread)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_single_empty_spread() {
        let diags = lint("var x = [...[]];");
        assert_eq!(diags.len(), 1, "[...[]] should be flagged");
    }

    #[test]
    fn test_flags_empty_spread_with_other_elements() {
        let diags = lint("var x = [...arr, ...[]];");
        assert_eq!(diags.len(), 1, "[...arr, ...[]] should be flagged");
    }

    #[test]
    fn test_allows_non_empty_spread() {
        let diags = lint("var x = [...[1, 2]];");
        assert!(diags.is_empty(), "[...[1, 2]] should not be flagged");
    }

    #[test]
    fn test_allows_spread_variable() {
        let diags = lint("var x = [...arr];");
        assert!(diags.is_empty(), "[...arr] should not be flagged");
    }

    #[test]
    fn test_allows_empty_array() {
        let diags = lint("var x = [];");
        assert!(diags.is_empty(), "empty array should not be flagged");
    }
}
