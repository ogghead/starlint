//! Rule: `jest/no-restricted-jest-methods`
//!
//! Warn when restricted Jest methods are used (e.g., `jest.advanceTimersByTime`).

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-restricted-jest-methods";

/// Default restricted `jest.*` methods.
const RESTRICTED_METHODS: &[&str] = &[
    "advanceTimersByTime",
    "advanceTimersByTimeAsync",
    "advanceTimersToNextTimer",
    "advanceTimersToNextTimerAsync",
    "clearAllTimers",
    "retryTimes",
];

/// Flags usage of restricted `jest.*` methods.
#[derive(Debug)]
pub struct NoRestrictedJestMethods;

impl NativeRule for NoRestrictedJestMethods {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow restricted Jest methods".to_owned(),
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

        // Match `jest.<method>(...)` pattern
        let method_name = match &call.callee {
            Expression::StaticMemberExpression(member) => {
                if matches!(&member.object, Expression::Identifier(id) if id.name.as_str() == "jest")
                {
                    member.property.name.as_str()
                } else {
                    return;
                }
            }
            _ => return,
        };

        if RESTRICTED_METHODS.contains(&method_name) {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!("`jest.{method_name}` is restricted"),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoRestrictedJestMethods)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_restricted_method() {
        let diags = lint("jest.advanceTimersByTime(1000);");
        assert_eq!(
            diags.len(),
            1,
            "`jest.advanceTimersByTime` should be flagged"
        );
    }

    #[test]
    fn test_flags_retry_times() {
        let diags = lint("jest.retryTimes(3);");
        assert_eq!(diags.len(), 1, "`jest.retryTimes` should be flagged");
    }

    #[test]
    fn test_allows_unrestricted_method() {
        let diags = lint("jest.fn();");
        assert!(
            diags.is_empty(),
            "`jest.fn` should not be flagged as it is not restricted"
        );
    }
}
