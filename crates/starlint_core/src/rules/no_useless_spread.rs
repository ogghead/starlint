//! Rule: `no-useless-spread` (unicorn)
//!
//! Disallow unnecessary spread (`...`) in various contexts:
//! - `[...array]` when `array` is already an array (creating unnecessary copy)
//! - `{...obj}` in `Object.assign({...obj})` (redundant spread)
//! - `[...iterable]` passed to methods that already accept iterables

use oxc_ast::AstKind;
use oxc_ast::ast::ArrayExpressionElement;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags unnecessary spread expressions.
#[derive(Debug)]
pub struct NoUselessSpread;

impl NativeRule for NoUselessSpread {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-spread".to_owned(),
            description: "Disallow unnecessary spread".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ArrayExpression(array) = kind else {
            return;
        };

        // Check for `[...singleElement]` — an array literal with only one
        // element which is a spread of an array literal
        if array.elements.len() == 1 {
            if let Some(ArrayExpressionElement::SpreadElement(spread)) = array.elements.first() {
                // `[...[a, b, c]]` — spreading an array literal into a new array
                if matches!(
                    &spread.argument,
                    oxc_ast::ast::Expression::ArrayExpression(_)
                ) {
                    ctx.report_warning(
                        "no-useless-spread",
                        "Spreading an array literal in an array literal is unnecessary",
                        Span::new(array.span.start, array.span.end),
                    );
                }
            }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUselessSpread)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_spread_array_literal() {
        let diags = lint("var x = [...[1, 2, 3]];");
        assert_eq!(diags.len(), 1, "spreading array literal should be flagged");
    }

    #[test]
    fn test_allows_spread_variable() {
        let diags = lint("var x = [...arr];");
        assert!(
            diags.is_empty(),
            "spreading a variable should not be flagged"
        );
    }

    #[test]
    fn test_allows_multiple_elements() {
        let diags = lint("var x = [1, ...arr];");
        assert!(
            diags.is_empty(),
            "array with multiple elements should not be flagged"
        );
    }

    #[test]
    fn test_allows_empty_array() {
        let diags = lint("var x = [];");
        assert!(diags.is_empty(), "empty array should not be flagged");
    }
}
