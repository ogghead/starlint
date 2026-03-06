//! Rule: `consistent-date-clone`
//!
//! Flag `new Date(date.getTime())` — prefer `new Date(date)` for cloning
//! dates. The `getTime()` call is unnecessary when passing to the `Date`
//! constructor.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `new Date(d.getTime())` — prefer `new Date(d)`.
#[derive(Debug)]
pub struct ConsistentDateClone;

impl NativeRule for ConsistentDateClone {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "consistent-date-clone".to_owned(),
            description: "Prefer `new Date(date)` over `new Date(date.getTime())`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::NewExpression])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::NewExpression(new_expr) = kind else {
            return;
        };

        // Check callee is `Date`
        let Expression::Identifier(callee_id) = &new_expr.callee else {
            return;
        };
        if callee_id.name.as_str() != "Date" {
            return;
        }

        // Must have exactly one argument
        if new_expr.arguments.len() != 1 {
            return;
        }

        let Some(first_arg) = new_expr.arguments.first() else {
            return;
        };

        // The argument must be a call expression (not a spread)
        let Argument::CallExpression(call) = first_arg else {
            return;
        };

        // The call must be `.getTime()` with no arguments
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "getTime" {
            return;
        }

        if !call.arguments.is_empty() {
            return;
        }

        // Fix: replace the argument `d.getTime()` with just `d`
        let source = ctx.source_text();
        let obj_span = member.object.span();
        let obj_start = obj_span.start as usize;
        let obj_end = obj_span.end as usize;
        let obj_text = source.get(obj_start..obj_end).unwrap_or("").to_owned();

        // Replace the entire first argument (the call expression) with just the object
        let arg_span = first_arg.span();
        let fix = (!obj_text.is_empty()).then(|| Fix {
            kind: FixKind::SafeFix,
            message: format!("Replace `{obj_text}.getTime()` with `{obj_text}`"),
            edits: vec![Edit {
                span: Span::new(arg_span.start, arg_span.end),
                replacement: obj_text.clone(),
            }],
            is_snippet: false,
        });

        ctx.report(Diagnostic {
            rule_name: "consistent-date-clone".to_owned(),
            message: "Use `new Date(date)` instead of `new Date(date.getTime())`".to_owned(),
            span: Span::new(new_expr.span.start, new_expr.span.end),
            severity: Severity::Warning,
            help: Some(format!("Replace `{obj_text}.getTime()` with `{obj_text}`")),
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ConsistentDateClone)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_date_get_time_clone() {
        let diags = lint("var d2 = new Date(d.getTime());");
        assert_eq!(diags.len(), 1, "new Date(d.getTime()) should be flagged");
    }

    #[test]
    fn test_allows_date_direct_clone() {
        let diags = lint("var d2 = new Date(d);");
        assert!(diags.is_empty(), "new Date(d) should not be flagged");
    }

    #[test]
    fn test_allows_date_no_args() {
        let diags = lint("var d = new Date();");
        assert!(diags.is_empty(), "new Date() should not be flagged");
    }

    #[test]
    fn test_allows_date_multiple_args() {
        let diags = lint("var d = new Date(2024, 0, 1);");
        assert!(diags.is_empty(), "new Date(y, m, d) should not be flagged");
    }

    #[test]
    fn test_allows_non_date_constructor() {
        let diags = lint("var x = new Foo(d.getTime());");
        assert!(
            diags.is_empty(),
            "non-Date constructor should not be flagged"
        );
    }
}
