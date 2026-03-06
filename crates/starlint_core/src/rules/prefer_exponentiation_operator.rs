//! Rule: `prefer-exponentiation-operator`
//!
//! Disallow the use of `Math.pow()` in favor of the `**` operator.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
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
            let fix = call
                .arguments
                .first()
                .zip(call.arguments.get(1))
                .map(|(first, second)| {
                    let source = ctx.source_text();
                    let f_start = usize::try_from(first.span().start).unwrap_or(0);
                    let f_end = usize::try_from(first.span().end).unwrap_or(0);
                    let s_start = usize::try_from(second.span().start).unwrap_or(0);
                    let s_end = usize::try_from(second.span().end).unwrap_or(0);
                    let first_text = source.get(f_start..f_end).unwrap_or("");
                    let second_text = source.get(s_start..s_end).unwrap_or("");
                    Fix {
                        kind: FixKind::SuggestionFix,
                        message: "Use `**` operator".to_owned(),
                        edits: vec![Edit {
                            span: Span::new(call.span.start, call.span.end),
                            replacement: format!("{first_text} ** {second_text}"),
                        }],
                        is_snippet: false,
                    }
                });

            ctx.report(Diagnostic {
                rule_name: "prefer-exponentiation-operator".to_owned(),
                message: "Use the `**` operator instead of `Math.pow()`".to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some("Replace with `**` operator".to_owned()),
                fix,
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
