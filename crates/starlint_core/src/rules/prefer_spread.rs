//! Rule: `prefer-spread`
//!
//! Require spread operator instead of `.apply()`. `foo.apply(null, args)`
//! should be written as `foo(...args)`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.apply()` calls that could use spread syntax.
#[derive(Debug)]
pub struct PreferSpread;

impl NativeRule for PreferSpread {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-spread".to_owned(),
            description: "Require spread operator instead of `.apply()`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "apply" {
            return;
        }

        // Only autofix `fn.apply(null, args)` or `fn.apply(undefined, args)` patterns
        let source = ctx.source_text();
        let obj_span = member.object.span();
        let fn_text = &source[obj_span.start as usize..obj_span.end as usize];

        // Try to extract autofix for the 2-arg pattern: fn.apply(null/undefined, args)
        let fix = if call.arguments.len() == 2 {
            let first_arg = call.arguments.first();
            let second_arg = call.arguments.get(1);
            let is_null_or_undefined = first_arg.is_some_and(|a| {
                let text = &source[a.span().start as usize..a.span().end as usize];
                text == "null" || text == "undefined"
            });
            if is_null_or_undefined {
                second_arg.map(|args_arg| {
                    let args_text =
                        &source[args_arg.span().start as usize..args_arg.span().end as usize];
                    let replacement = format!("{fn_text}(...{args_text})");
                    Fix {
                        kind: FixKind::SuggestionFix,
                        message: format!("Replace with `{replacement}`"),
                        edits: vec![Edit {
                            span: Span::new(call.span.start, call.span.end),
                            replacement,
                        }],
                        is_snippet: false,
                    }
                })
            } else {
                None
            }
        } else {
            None
        };

        let message = "Use the spread operator instead of `.apply()`".to_owned();
        ctx.report(Diagnostic {
            rule_name: "prefer-spread".to_owned(),
            message: message.clone(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some(message),
            fix,
            labels: vec![],
        });
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferSpread)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_apply() {
        let diags = lint("foo.apply(null, args);");
        assert_eq!(diags.len(), 1, ".apply() should be flagged");
    }

    #[test]
    fn test_allows_spread() {
        let diags = lint("foo(...args);");
        assert!(diags.is_empty(), "spread operator should not be flagged");
    }

    #[test]
    fn test_allows_normal_call() {
        let diags = lint("foo(1, 2);");
        assert!(diags.is_empty(), "normal call should not be flagged");
    }
}
