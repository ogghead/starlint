//! Rule: `symbol-description`
//!
//! Require a description when creating a `Symbol`. Providing a description
//! makes debugging easier since it appears in `toString()`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `Symbol()` calls without a description argument.
#[derive(Debug)]
pub struct SymbolDescription;

impl NativeRule for SymbolDescription {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "symbol-description".to_owned(),
            description: "Require a description when creating a Symbol".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let Expression::Identifier(id) = &call.callee else {
            return;
        };

        if id.name.as_str() != "Symbol" {
            return;
        }

        if call.arguments.is_empty() {
            ctx.report_warning(
                "symbol-description",
                "Provide a description for `Symbol()` to aid debugging",
                Span::new(call.span.start, call.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(SymbolDescription)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_symbol_without_description() {
        let diags = lint("var s = Symbol();");
        assert_eq!(
            diags.len(),
            1,
            "Symbol() without description should be flagged"
        );
    }

    #[test]
    fn test_allows_symbol_with_description() {
        let diags = lint("var s = Symbol('mySymbol');");
        assert!(
            diags.is_empty(),
            "Symbol with description should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_symbol_call() {
        let diags = lint("var x = foo();");
        assert!(diags.is_empty(), "non-Symbol call should not be flagged");
    }
}
