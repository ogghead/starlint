//! Rule: `no-map-spread`
//!
//! Disallow spreading a `Map` in an object literal (`{...new Map()}`).
//! Map entries don't spread into object properties — the result is an empty
//! object, which is almost certainly a bug. Array spread (`[...new Map()]`)
//! is fine because it yields the Map's entries as `[key, value]` pairs.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, ObjectPropertyKind};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `{...new Map()}` in object literals.
#[derive(Debug)]
pub struct NoMapSpread;

impl NativeRule for NoMapSpread {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-map-spread".to_owned(),
            description: "Disallow spreading a Map in an object literal".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ObjectExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ObjectExpression(obj) = kind else {
            return;
        };

        for property in &obj.properties {
            let ObjectPropertyKind::SpreadProperty(spread) = property else {
                continue;
            };

            // Check if the spread argument is `new Map(...)`.
            let Expression::NewExpression(new_expr) = &spread.argument else {
                continue;
            };

            if let Expression::Identifier(id) = &new_expr.callee {
                if id.name.as_str() == "Map" {
                    ctx.report_error(
                        "no-map-spread",
                        "Spreading a Map into an object literal produces an empty object — Map entries are not object properties",
                        Span::new(spread.span.start, spread.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoMapSpread)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_spread_new_map() {
        let diags = lint("const x = { ...new Map() };");
        assert_eq!(diags.len(), 1, "spreading new Map() should be flagged");
    }

    #[test]
    fn test_flags_spread_new_map_with_args() {
        let diags = lint("const x = { ...new Map([['a', 1]]) };");
        assert_eq!(
            diags.len(),
            1,
            "spreading new Map(...) with arguments should be flagged"
        );
    }

    #[test]
    fn test_allows_spread_plain_object() {
        let diags = lint("const x = { ...obj };");
        assert!(
            diags.is_empty(),
            "spreading a plain object should not be flagged"
        );
    }

    #[test]
    fn test_allows_spread_new_set() {
        let diags = lint("const x = { ...new Set() };");
        assert!(
            diags.is_empty(),
            "spreading new Set() should not be flagged (only Map is checked)"
        );
    }

    #[test]
    fn test_allows_array_spread_map() {
        let diags = lint("const x = [...new Map()];");
        assert!(
            diags.is_empty(),
            "array spread of new Map() should not be flagged"
        );
    }

    #[test]
    fn test_allows_normal_object_properties() {
        let diags = lint("const x = { a: 1, b: 2 };");
        assert!(
            diags.is_empty(),
            "normal object properties should not be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_map_spreads() {
        let diags = lint("const x = { ...new Map(), ...new Map() };");
        assert_eq!(
            diags.len(),
            2,
            "two Map spreads should produce two diagnostics"
        );
    }
}
