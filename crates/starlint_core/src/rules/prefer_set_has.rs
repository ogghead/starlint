//! Rule: `prefer-set-has`
//!
//! Prefer `Set#has()` over `Array#includes()` when checking membership in
//! an array literal. Array literals used as lookup tables should be converted
//! to a `Set` for O(1) lookups instead of O(n) scans.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.includes()` calls on array literals.
#[derive(Debug)]
pub struct PreferSetHas;

impl NativeRule for PreferSetHas {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-set-has".to_owned(),
            description: "Prefer `Set#has()` over `Array#includes()` for array literal lookups"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
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

        if member.property.name.as_str() != "includes" {
            return;
        }

        // Must have exactly one argument (the search value)
        if call.arguments.len() != 1 {
            return;
        }

        // The first argument must not be a spread element
        if let Some(Argument::SpreadElement(_)) = call.arguments.first() {
            return;
        }

        // The object must be an array literal
        if !matches!(&member.object, Expression::ArrayExpression(_)) {
            return;
        }

        ctx.report_warning(
            "prefer-set-has",
            "Use `new Set([...]).has()` instead of `[...].includes()` for better performance",
            Span::new(call.span.start, call.span.end),
        );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferSetHas)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_array_literal_includes() {
        let diags = lint("['a', 'b', 'c'].includes(x);");
        assert_eq!(
            diags.len(),
            1,
            "array literal .includes() should be flagged"
        );
    }

    #[test]
    fn test_flags_numeric_array_includes() {
        let diags = lint("[1, 2, 3].includes(val);");
        assert_eq!(
            diags.len(),
            1,
            "numeric array literal .includes() should be flagged"
        );
    }

    #[test]
    fn test_allows_variable_includes() {
        let diags = lint("arr.includes(x);");
        assert!(
            diags.is_empty(),
            "variable .includes() should not be flagged"
        );
    }

    #[test]
    fn test_allows_string_includes() {
        let diags = lint("str.includes('sub');");
        assert!(diags.is_empty(), "string .includes() should not be flagged");
    }

    #[test]
    fn test_allows_set_has() {
        let diags = lint("new Set(['a']).has(x);");
        assert!(diags.is_empty(), "Set.has() should not be flagged");
    }

    #[test]
    fn test_allows_includes_with_from_index() {
        let diags = lint("['a', 'b'].includes(x, 1);");
        assert!(
            diags.is_empty(),
            ".includes() with fromIndex should not be flagged"
        );
    }
}
