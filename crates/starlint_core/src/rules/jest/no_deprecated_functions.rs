//! Rule: `jest/no-deprecated-functions`
//!
//! Error when deprecated Jest functions are used.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-deprecated-functions";

/// Deprecated `jest.*` methods and their replacements.
const DEPRECATED: &[(&str, &str)] = &[
    ("resetModuleRegistry", "jest.resetModules"),
    ("addMatchers", "expect.extend"),
    ("runTimersToTime", "jest.advanceTimersByTime"),
    ("genMockFromModule", "jest.createMockFromModule"),
];

/// Flags usage of deprecated Jest functions.
#[derive(Debug)]
pub struct NoDeprecatedFunctions;

impl NativeRule for NoDeprecatedFunctions {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow deprecated Jest functions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
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

        for &(deprecated_name, replacement) in DEPRECATED {
            if method_name == deprecated_name {
                ctx.report_error(
                    RULE_NAME,
                    &format!(
                        "`jest.{deprecated_name}` is deprecated — use `{replacement}` instead"
                    ),
                    Span::new(call.span.start, call.span.end),
                );
                return;
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDeprecatedFunctions)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_reset_module_registry() {
        let diags = lint("jest.resetModuleRegistry();");
        assert_eq!(
            diags.len(),
            1,
            "`jest.resetModuleRegistry` should be flagged as deprecated"
        );
    }

    #[test]
    fn test_flags_add_matchers() {
        let diags = lint("jest.addMatchers({});");
        assert_eq!(
            diags.len(),
            1,
            "`jest.addMatchers` should be flagged as deprecated"
        );
    }

    #[test]
    fn test_allows_modern_methods() {
        let diags = lint("jest.resetModules();");
        assert!(
            diags.is_empty(),
            "`jest.resetModules` is not deprecated and should not be flagged"
        );
    }
}
