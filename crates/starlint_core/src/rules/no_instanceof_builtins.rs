//! Rule: `no-instanceof-builtins` (unicorn)
//!
//! Prefer builtin type-checking methods over `instanceof` for built-in types.
//! `instanceof Array` doesn't work across realms (iframes, workers).
//! Use `Array.isArray()` instead.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `instanceof` checks on built-in types.
#[derive(Debug)]
pub struct NoInstanceofBuiltins;

/// Built-in types that have better type-checking alternatives.
const BUILTIN_TYPES: &[(&str, &str)] = &[
    ("Array", "Use `Array.isArray()` instead"),
    (
        "ArrayBuffer",
        "Use `ArrayBuffer.isView()` or check constructor",
    ),
    (
        "Error",
        "Use `error instanceof Error` is OK, but consider `cause` chain",
    ),
];

impl NativeRule for NoInstanceofBuiltins {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-instanceof-builtins".to_owned(),
            description: "Prefer builtin type-checking methods over instanceof".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::BinaryExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::BinaryExpression(bin) = kind else {
            return;
        };

        if !matches!(bin.operator, oxc_ast::ast::BinaryOperator::Instanceof) {
            return;
        }

        let Expression::Identifier(right_id) = &bin.right else {
            return;
        };

        let name = right_id.name.as_str();
        if let Some((_builtin, suggestion)) = BUILTIN_TYPES.iter().find(|(b, _)| *b == name) {
            ctx.report_warning(
                "no-instanceof-builtins",
                &format!(
                    "Avoid `instanceof {name}` which doesn't work across realms. {suggestion}"
                ),
                Span::new(bin.span.start, bin.span.end),
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoInstanceofBuiltins)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_instanceof_array() {
        let diags = lint("if (x instanceof Array) {}");
        assert_eq!(diags.len(), 1, "instanceof Array should be flagged");
    }

    #[test]
    fn test_allows_instanceof_custom() {
        let diags = lint("if (x instanceof MyClass) {}");
        assert!(
            diags.is_empty(),
            "instanceof custom class should not be flagged"
        );
    }

    #[test]
    fn test_allows_array_isarray() {
        let diags = lint("if (Array.isArray(x)) {}");
        assert!(diags.is_empty(), "Array.isArray should not be flagged");
    }
}
