//! Rule: `prefer-array-flat-map` (unicorn)
//!
//! Prefer `.flatMap()` over `.map().flat()`. Using `flatMap` is more
//! concise and performs the operation in a single pass.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.map(...).flat()` chains that should use `.flatMap()`.
#[derive(Debug)]
pub struct PreferArrayFlatMap;

impl NativeRule for PreferArrayFlatMap {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-array-flat-map".to_owned(),
            description: "Prefer .flatMap() over .map().flat()".to_owned(),
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

        // Check for `.flat()` call
        let Expression::StaticMemberExpression(flat_member) = &call.callee else {
            return;
        };

        if flat_member.property.name != "flat" {
            return;
        }

        // `.flat()` should have 0 args or 1 arg that's the literal `1`
        let is_flat_one = call.arguments.is_empty()
            || (call.arguments.len() == 1
                && call.arguments.first().is_some_and(|arg| {
                    matches!(
                        arg,
                        oxc_ast::ast::Argument::NumericLiteral(n) if (n.value - 1.0).abs() < f64::EPSILON
                    )
                }));

        if !is_flat_one {
            return;
        }

        // Check if the object is a `.map(...)` call
        let Expression::CallExpression(map_call) = &flat_member.object else {
            return;
        };

        let Expression::StaticMemberExpression(map_member) = &map_call.callee else {
            return;
        };

        if map_member.property.name == "map" {
            let call_span = Span::new(call.span.start, call.span.end);
            // Fix: replace `map` with `flatMap` and remove `.flat()` suffix.
            // We need the source of the `.map(...)` call (which ends at map_call.span.end)
            // and rename `map` to `flatMap` — effectively keeping everything up to the
            // end of the map call and replacing the outer `.flat()`.
            let map_prop_span =
                Span::new(map_member.property.span.start, map_member.property.span.end);
            // The overall fix: rename `.map` to `.flatMap` and replace the outer
            // `.flat()` call span with just the map call result.
            // Strategy: two edits — rename `map` -> `flatMap`, and delete `.flat()` / `.flat(1)`
            // The `.flat(...)` portion starts right after map_call ends.
            let flat_suffix_span = Span::new(map_call.span.end, call.span.end);
            ctx.report(Diagnostic {
                rule_name: "prefer-array-flat-map".to_owned(),
                message: "Prefer `.flatMap()` over `.map().flat()`".to_owned(),
                span: call_span,
                severity: Severity::Warning,
                help: Some("Use `.flatMap()` instead".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: "Replace `.map().flat()` with `.flatMap()`".to_owned(),
                    edits: vec![
                        Edit {
                            span: map_prop_span,
                            replacement: "flatMap".to_owned(),
                        },
                        Edit {
                            span: flat_suffix_span,
                            replacement: String::new(),
                        },
                    ],
                    is_snippet: false,
                }),
                labels: vec![],
            });
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferArrayFlatMap)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_map_flat() {
        let diags = lint("arr.map(x => [x]).flat();");
        assert_eq!(diags.len(), 1, ".map().flat() should be flagged");
    }

    #[test]
    fn test_flags_map_flat_one() {
        let diags = lint("arr.map(x => [x]).flat(1);");
        assert_eq!(diags.len(), 1, ".map().flat(1) should be flagged");
    }

    #[test]
    fn test_allows_flat_map() {
        let diags = lint("arr.flatMap(x => [x]);");
        assert!(diags.is_empty(), "flatMap should not be flagged");
    }

    #[test]
    fn test_allows_map_flat_deep() {
        let diags = lint("arr.map(x => [x]).flat(2);");
        assert!(
            diags.is_empty(),
            ".map().flat(2) should not be flagged (deep flat)"
        );
    }

    #[test]
    fn test_allows_flat_alone() {
        let diags = lint("arr.flat();");
        assert!(diags.is_empty(), ".flat() alone should not be flagged");
    }
}
