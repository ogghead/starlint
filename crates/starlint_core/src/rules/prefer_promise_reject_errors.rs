//! Rule: `prefer-promise-reject-errors`
//!
//! Require using Error objects as Promise rejection reasons.
//! `Promise.reject('error')` should be `Promise.reject(new Error('error'))`.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `Promise.reject()` calls with non-Error arguments.
#[derive(Debug)]
pub struct PreferPromiseRejectErrors;

impl NativeRule for PreferPromiseRejectErrors {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-promise-reject-errors".to_owned(),
            description: "Require using Error objects as Promise rejection reasons".to_owned(),
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

        // Check for Promise.reject(...)
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "reject" {
            return;
        }

        if !matches!(&member.object, Expression::Identifier(id) if id.name.as_str() == "Promise") {
            return;
        }

        // Check the first argument — flag if it's a literal (not an Error)
        if let Some(first_arg) = call.arguments.first() {
            let is_literal_rejection = matches!(
                first_arg,
                Argument::StringLiteral(_)
                    | Argument::NumericLiteral(_)
                    | Argument::BooleanLiteral(_)
                    | Argument::NullLiteral(_)
            );

            if is_literal_rejection {
                #[allow(clippy::as_conversions)]
                let fix = {
                    let arg_span = first_arg.span();
                    let source = ctx.source_text();
                    source
                        .get(arg_span.start as usize..arg_span.end as usize)
                        .map(|arg_text| {
                            let replacement = format!("new Error({arg_text})");
                            Fix {
                                message: format!("Replace with `{replacement}`"),
                                edits: vec![Edit {
                                    span: Span::new(arg_span.start, arg_span.end),
                                    replacement,
                                }],
                                is_snippet: false,
                            }
                        })
                };

                ctx.report(Diagnostic {
                    rule_name: "prefer-promise-reject-errors".to_owned(),
                    message: "Expected the Promise rejection reason to be an Error".to_owned(),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Warning,
                    help: Some(
                        "Wrap the rejection reason in `new Error(...)` for better stack traces"
                            .to_owned(),
                    ),
                    fix,
                    labels: vec![],
                });
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferPromiseRejectErrors)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_string_rejection() {
        let diags = lint("Promise.reject('error');");
        assert_eq!(diags.len(), 1, "string rejection should be flagged");
    }

    #[test]
    fn test_allows_error_rejection() {
        let diags = lint("Promise.reject(new Error('error'));");
        assert!(diags.is_empty(), "Error rejection should not be flagged");
    }

    #[test]
    fn test_allows_variable_rejection() {
        let diags = lint("Promise.reject(err);");
        assert!(diags.is_empty(), "variable rejection should not be flagged");
    }
}
