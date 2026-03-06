//! Rule: `no-useless-call`
//!
//! Disallow unnecessary `.call()` and `.apply()`. Using `foo.call(thisArg)`
//! when `thisArg` is the receiver is equivalent to just `foo()` and the
//! `.call()`/`.apply()` is unnecessary.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags unnecessary `.call()` and `.apply()` invocations.
#[derive(Debug)]
pub struct NoUselessCall;

impl NativeRule for NoUselessCall {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-call".to_owned(),
            description: "Disallow unnecessary `.call()` and `.apply()`".to_owned(),
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

        let method = member.property.name.as_str();
        if method != "call" && method != "apply" {
            return;
        }

        // Must have at least one argument (thisArg)
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        // Check if thisArg is `null` or `undefined` — this means the function
        // is called without a specific this binding, which is useless
        let is_null_or_undefined = match first_arg {
            Argument::NullLiteral(_) => true,
            Argument::Identifier(id) => id.name.as_str() == "undefined",
            _ => false,
        };

        if is_null_or_undefined {
            // Build fix: foo.call(null, a, b) → foo(a, b)
            #[allow(clippy::as_conversions)]
            let fix = {
                let source = ctx.source_text();
                let obj_span = member.object.span();
                let obj_text = source
                    .get(obj_span.start as usize..obj_span.end as usize)
                    .unwrap_or("");

                // Collect remaining args (skip thisArg)
                let remaining_args: Vec<&str> = call
                    .arguments
                    .iter()
                    .skip(1)
                    .filter_map(|arg| {
                        let s = arg.span();
                        source.get(s.start as usize..s.end as usize)
                    })
                    .collect();

                let args_str = remaining_args.join(", ");
                let replacement = format!("{obj_text}({args_str})");
                Some(Fix {
                    message: format!("Replace with `{replacement}`"),
                    edits: vec![Edit {
                        span: Span::new(call.span.start, call.span.end),
                        replacement,
                    }],
                })
            };

            ctx.report(Diagnostic {
                rule_name: "no-useless-call".to_owned(),
                message: format!("Unnecessary `.{method}()` — call the function directly instead"),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some("Remove `.call()`/`.apply()` and call the function directly".to_owned()),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUselessCall)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_call_with_null() {
        let diags = lint("foo.call(null, 1, 2);");
        assert_eq!(diags.len(), 1, "foo.call(null, ...) should be flagged");
    }

    #[test]
    fn test_flags_apply_with_undefined() {
        let diags = lint("foo.apply(undefined, [1, 2]);");
        assert_eq!(
            diags.len(),
            1,
            "foo.apply(undefined, ...) should be flagged"
        );
    }

    #[test]
    fn test_allows_call_with_this_arg() {
        let diags = lint("foo.call(obj, 1, 2);");
        assert!(diags.is_empty(), "foo.call(obj, ...) should not be flagged");
    }

    #[test]
    fn test_allows_normal_call() {
        let diags = lint("foo(1, 2);");
        assert!(
            diags.is_empty(),
            "normal function call should not be flagged"
        );
    }
}
