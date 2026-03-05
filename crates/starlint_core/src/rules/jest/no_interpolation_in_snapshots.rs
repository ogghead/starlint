//! Rule: `jest/no-interpolation-in-snapshots`
//!
//! Error when template literals with expressions are used in inline snapshot arguments.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-interpolation-in-snapshots";

/// Snapshot matcher names that should not receive interpolated template literals.
const SNAPSHOT_MATCHERS: &[&str] = &[
    "toMatchInlineSnapshot",
    "toThrowErrorMatchingInlineSnapshot",
];

/// Flags template literals with expressions used as inline snapshot arguments.
#[derive(Debug)]
pub struct NoInterpolationInSnapshots;

impl NativeRule for NoInterpolationInSnapshots {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow template literal interpolation in inline snapshots".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
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

        // Check for `.toMatchInlineSnapshot(...)` or `.toThrowErrorMatchingInlineSnapshot(...)`
        let is_snapshot_matcher = match &call.callee {
            Expression::StaticMemberExpression(member) => {
                SNAPSHOT_MATCHERS.contains(&member.property.name.as_str())
            }
            _ => false,
        };

        if !is_snapshot_matcher {
            return;
        }

        // Check if any argument is a template literal with expressions
        for arg in &call.arguments {
            if let Some(Expression::TemplateLiteral(tmpl)) = arg.as_expression() {
                if !tmpl.expressions.is_empty() {
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: "Do not use template literal interpolation in inline snapshots — snapshots should be static strings".to_owned(),
                        span: Span::new(call.span.start, call.span.end),
                        severity: Severity::Error,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                    return;
                }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoInterpolationInSnapshots)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_interpolated_snapshot() {
        let source = "expect(value).toMatchInlineSnapshot(`value is ${x}`);";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "template literal with interpolation in snapshot should be flagged"
        );
    }

    #[test]
    fn test_flags_interpolated_throw_snapshot() {
        let source = "expect(fn).toThrowErrorMatchingInlineSnapshot(`error: ${msg}`);";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "interpolation in toThrowErrorMatchingInlineSnapshot should be flagged"
        );
    }

    #[test]
    fn test_allows_static_template_literal() {
        let source = "expect(value).toMatchInlineSnapshot(`static value`);";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "static template literal without interpolation should not be flagged"
        );
    }
}
