//! Rule: `prefer-array-flat`
//!
//! Prefer `Array.prototype.flat()` over legacy flattening patterns.
//! Flags `.reduce()` calls whose callback body contains `.concat()`,
//! which is a common pattern for flattening arrays before `.flat()` existed.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.reduce()` calls that likely flatten arrays using `.concat()`.
#[derive(Debug)]
pub struct PreferArrayFlat;

impl NativeRule for PreferArrayFlat {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-array-flat".to_owned(),
            description: "Prefer `.flat()` over `.reduce()` with `.concat()`".to_owned(),
            category: Category::Style,
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

        // Must be a `.reduce()` call.
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };
        if member.property.name.as_str() != "reduce" {
            return;
        }

        // Check source text of the call for `.concat(` — a simple heuristic
        // that catches the common `(a, b) => a.concat(b)` pattern without
        // deep AST inspection of the callback body.
        let start = usize::try_from(call.span.start).unwrap_or(0);
        let end = usize::try_from(call.span.end).unwrap_or(0);
        let Some(raw) = ctx.source_text().get(start..end) else {
            return;
        };

        if !raw.contains(".concat(") {
            return;
        }

        // Autofix: replace `reduce(...)` with `flat()` (from property name to end of call)
        ctx.report(Diagnostic {
            rule_name: "prefer-array-flat".to_owned(),
            message: "Prefer `.flat()` over `.reduce()` with `.concat()` for flattening arrays"
                .to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some("Replace with `.flat()`".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Replace with `.flat()`".to_owned(),
                edits: vec![Edit {
                    span: Span::new(member.property.span.start, call.span.end),
                    replacement: "flat()".to_owned(),
                }],
                is_snippet: false,
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferArrayFlat)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_reduce_concat() {
        let diags = lint("const flat = arr.reduce((a, b) => a.concat(b), []);");
        assert_eq!(diags.len(), 1, "should flag .reduce() with .concat()");
    }

    #[test]
    fn test_allows_reduce_without_concat() {
        let diags = lint("const sum = arr.reduce((a, b) => a + b, 0);");
        assert!(
            diags.is_empty(),
            ".reduce() without .concat() should not be flagged"
        );
    }

    #[test]
    fn test_allows_flat() {
        let diags = lint("const flat = arr.flat();");
        assert!(diags.is_empty(), ".flat() should not be flagged");
    }
}
