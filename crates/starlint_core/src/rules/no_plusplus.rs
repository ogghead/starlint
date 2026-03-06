//! Rule: `no-plusplus`
//!
//! Disallow the unary operators `++` and `--`. These can be confusing due
//! to automatic semicolon insertion and can be replaced with `+= 1`/`-= 1`.

use oxc_ast::AstKind;
use oxc_ast::ast::UpdateOperator;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `++` and `--` unary operators.
#[derive(Debug)]
pub struct NoPlusplus;

impl NativeRule for NoPlusplus {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-plusplus".to_owned(),
            description: "Disallow the unary operators `++` and `--`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::UpdateExpression])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::UpdateExpression(update) = kind else {
            return;
        };

        let op_str = match update.operator {
            UpdateOperator::Increment => "++",
            UpdateOperator::Decrement => "--",
        };

        let assign_op = match update.operator {
            UpdateOperator::Increment => "+= 1",
            UpdateOperator::Decrement => "-= 1",
        };

        // Extract the argument source text for the fix
        let source = ctx.source_text();
        let arg_start = update.argument.span().start as usize;
        let arg_end = update.argument.span().end as usize;
        let arg_text = source.get(arg_start..arg_end).unwrap_or("").to_owned();

        let replacement = format!("{arg_text} {assign_op}");
        let fix = (!arg_text.is_empty()).then(|| Fix {
            kind: FixKind::SuggestionFix,
            message: format!("Replace `{op_str}` with `{assign_op}`"),
            edits: vec![Edit {
                span: Span::new(update.span.start, update.span.end),
                replacement,
            }],
            is_snippet: false,
        });

        ctx.report(Diagnostic {
            rule_name: "no-plusplus".to_owned(),
            message: format!("Unary operator `{op_str}` used"),
            span: Span::new(update.span.start, update.span.end),
            severity: Severity::Warning,
            help: Some(format!("Use `{assign_op}` instead")),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoPlusplus)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_increment() {
        let diags = lint("x++;");
        assert_eq!(diags.len(), 1, "++ should be flagged");
    }

    #[test]
    fn test_flags_decrement() {
        let diags = lint("x--;");
        assert_eq!(diags.len(), 1, "-- should be flagged");
    }

    #[test]
    fn test_flags_prefix_increment() {
        let diags = lint("++x;");
        assert_eq!(diags.len(), 1, "prefix ++ should be flagged");
    }

    #[test]
    fn test_allows_plus_equal() {
        let diags = lint("x += 1;");
        assert!(diags.is_empty(), "+= 1 should not be flagged");
    }
}
