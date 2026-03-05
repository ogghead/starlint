//! Rule: `jest/prefer-called-with`
//!
//! Suggest `toHaveBeenCalledWith` over `toHaveBeenCalled`. Using the more
//! specific `toHaveBeenCalledWith` ensures mock functions are called with
//! the expected arguments, catching bugs where the right function is called
//! but with wrong parameters.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `toHaveBeenCalled()` / `toBeCalled()` in favor of `toHaveBeenCalledWith()`.
#[derive(Debug)]
pub struct PreferCalledWith;

impl NativeRule for PreferCalledWith {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-called-with".to_owned(),
            description: "Suggest using `toHaveBeenCalledWith()` over `toHaveBeenCalled()`"
                .to_owned(),
            category: Category::Suggestion,
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
        let method = member.property.name.as_str();
        if method != "toHaveBeenCalled" && method != "toBeCalled" {
            return;
        }

        if !is_expect_chain(&member.object) {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "jest/prefer-called-with".to_owned(),
            message: format!(
                "Use `toHaveBeenCalledWith()` instead of `{method}()` for more specific assertions"
            ),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }
}

/// Check if an expression is an `expect(...)` call or a chain like
/// `expect(...).not`.
fn is_expect_chain(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::CallExpression(call) => {
            matches!(&call.callee, Expression::Identifier(id) if id.name.as_str() == "expect")
        }
        Expression::StaticMemberExpression(member) => is_expect_chain(&member.object),
        _ => false,
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferCalledWith)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_to_have_been_called() {
        let diags = lint("expect(mockFn).toHaveBeenCalled();");
        assert_eq!(diags.len(), 1, "`toHaveBeenCalled()` should be flagged");
    }

    #[test]
    fn test_flags_to_be_called() {
        let diags = lint("expect(mockFn).toBeCalled();");
        assert_eq!(diags.len(), 1, "`toBeCalled()` should be flagged");
    }

    #[test]
    fn test_allows_to_have_been_called_with() {
        let diags = lint("expect(mockFn).toHaveBeenCalledWith(1, 2);");
        assert!(
            diags.is_empty(),
            "`toHaveBeenCalledWith()` should not be flagged"
        );
    }
}
