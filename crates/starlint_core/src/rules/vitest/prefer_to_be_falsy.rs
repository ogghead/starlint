//! Rule: `vitest/prefer-to-be-falsy`
//!
//! Suggest `toBeFalsy()` over `toBe(false)`. The `toBeFalsy()` matcher is
//! more idiomatic in Vitest for checking falsy values and provides clearer
//! intent.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "vitest/prefer-to-be-falsy";

/// Suggest `toBeFalsy()` over `toBe(false)`.
#[derive(Debug)]
pub struct PreferToBeFalsy;

impl NativeRule for PreferToBeFalsy {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prefer `toBeFalsy()` over `toBe(false)`".to_owned(),
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

        // Match `.toBe(false)`.
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "toBe" {
            return;
        }

        if call.arguments.len() != 1 {
            return;
        }

        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        let is_false = matches!(first_arg, Argument::BooleanLiteral(lit) if !lit.value);

        if is_false {
            // Replace from the property name start to end of call: `toBe(false)` -> `toBeFalsy()`
            let fix_span = Span::new(member.property.span.start, call.span.end);
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Prefer `toBeFalsy()` over `toBe(false)`".to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some("Replace `toBe(false)` with `toBeFalsy()`".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Replace with `toBeFalsy()`".to_owned(),
                    edits: vec![Edit {
                        span: fix_span,
                        replacement: "toBeFalsy()".to_owned(),
                    }],
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferToBeFalsy)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_to_be_false() {
        let source = "expect(value).toBe(false);";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "`.toBe(false)` should be flagged");
    }

    #[test]
    fn test_allows_to_be_falsy() {
        let source = "expect(value).toBeFalsy();";
        let diags = lint(source);
        assert!(diags.is_empty(), "`.toBeFalsy()` should not be flagged");
    }

    #[test]
    fn test_allows_to_be_true() {
        let source = "expect(value).toBe(true);";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`.toBe(true)` should not be flagged by this rule"
        );
    }
}
