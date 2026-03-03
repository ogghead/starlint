//! Rule: `prefer-exponentiation-operator`
//!
//! Disallow the use of `Math.pow()` in favor of the `**` operator.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `Math.pow()` calls.
#[derive(Debug)]
pub struct PreferExponentiationOperator;

impl NativeRule for PreferExponentiationOperator {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-exponentiation-operator".to_owned(),
            description: "Disallow the use of `Math.pow` in favor of `**`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
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

        if member.property.name.as_str() != "pow" {
            return;
        }

        if matches!(&member.object, Expression::Identifier(id) if id.name.as_str() == "Math") {
            ctx.report_warning(
                "prefer-exponentiation-operator",
                "Use the `**` operator instead of `Math.pow()`",
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferExponentiationOperator)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_math_pow() {
        let diags = lint("var x = Math.pow(2, 3);");
        assert_eq!(diags.len(), 1, "Math.pow() should be flagged");
    }

    #[test]
    fn test_allows_exponentiation_operator() {
        let diags = lint("var x = 2 ** 3;");
        assert!(diags.is_empty(), "** operator should not be flagged");
    }

    #[test]
    fn test_allows_other_math_methods() {
        let diags = lint("var x = Math.floor(3.14);");
        assert!(diags.is_empty(), "other Math methods should not be flagged");
    }
}
