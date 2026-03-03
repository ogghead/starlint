//! Rule: `vitest/prefer-called-once`
//!
//! Suggest `toHaveBeenCalledOnce()` over `toHaveBeenCalledTimes(1)`.
//! The `toHaveBeenCalledOnce()` matcher is more readable and expressive
//! when asserting exactly one call.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "vitest/prefer-called-once";

/// Suggest `toHaveBeenCalledOnce()` over `toHaveBeenCalledTimes(1)`.
#[derive(Debug)]
pub struct PreferCalledOnce;

impl NativeRule for PreferCalledOnce {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prefer `toHaveBeenCalledOnce()` over `toHaveBeenCalledTimes(1)`"
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

        // Match `.toHaveBeenCalledTimes(1)`.
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "toHaveBeenCalledTimes" {
            return;
        }

        // Check that the single argument is the numeric literal `1`.
        if call.arguments.len() != 1 {
            return;
        }

        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        let is_one = match first_arg {
            Argument::NumericLiteral(lit) => {
                #[allow(clippy::float_cmp)]
                {
                    lit.value == 1.0
                }
            }
            _ => false,
        };

        if is_one {
            ctx.report_warning(
                RULE_NAME,
                "Prefer `toHaveBeenCalledOnce()` over `toHaveBeenCalledTimes(1)`",
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferCalledOnce)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_called_times_one() {
        let source = "expect(mock).toHaveBeenCalledTimes(1);";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "`toHaveBeenCalledTimes(1)` should be flagged"
        );
    }

    #[test]
    fn test_allows_called_times_other() {
        let source = "expect(mock).toHaveBeenCalledTimes(3);";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`toHaveBeenCalledTimes(3)` should not be flagged"
        );
    }

    #[test]
    fn test_allows_called_once() {
        let source = "expect(mock).toHaveBeenCalledOnce();";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`toHaveBeenCalledOnce()` should not be flagged"
        );
    }
}
