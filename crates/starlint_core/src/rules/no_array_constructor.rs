//! Rule: `no-array-constructor`
//!
//! Disallow `Array` constructors. Use array literal syntax `[]` instead.
//! `new Array(1, 2)` should be `[1, 2]`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `Array()` and `new Array()` with multiple arguments.
#[derive(Debug)]
pub struct NoArrayConstructor;

impl NativeRule for NoArrayConstructor {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-array-constructor".to_owned(),
            description: "Disallow `Array` constructor".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression, AstType::NewExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::NewExpression(new_expr) => {
                if matches!(&new_expr.callee, Expression::Identifier(id) if id.name.as_str() == "Array")
                    && new_expr.arguments.len() != 1
                {
                    let source = ctx.source_text();
                    let replacement = build_array_literal(&new_expr.arguments, source);
                    ctx.report(Diagnostic {
                        rule_name: "no-array-constructor".to_owned(),
                        message: "Use array literal `[]` instead of `Array` constructor".to_owned(),
                        span: Span::new(new_expr.span.start, new_expr.span.end),
                        severity: Severity::Warning,
                        help: Some("Replace with array literal".to_owned()),
                        fix: Some(Fix {
                            message: "Replace with array literal".to_owned(),
                            edits: vec![Edit {
                                span: Span::new(new_expr.span.start, new_expr.span.end),
                                replacement,
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                }
            }
            AstKind::CallExpression(call) => {
                if matches!(&call.callee, Expression::Identifier(id) if id.name.as_str() == "Array")
                    && call.arguments.len() != 1
                {
                    let source = ctx.source_text();
                    let replacement = build_array_literal(&call.arguments, source);
                    ctx.report(Diagnostic {
                        rule_name: "no-array-constructor".to_owned(),
                        message: "Use array literal `[]` instead of `Array` constructor".to_owned(),
                        span: Span::new(call.span.start, call.span.end),
                        severity: Severity::Warning,
                        help: Some("Replace with array literal".to_owned()),
                        fix: Some(Fix {
                            message: "Replace with array literal".to_owned(),
                            edits: vec![Edit {
                                span: Span::new(call.span.start, call.span.end),
                                replacement,
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                }
            }
            _ => {}
        }
    }
}

/// Build an array literal string from arguments.
fn build_array_literal(args: &[oxc_ast::ast::Argument<'_>], source: &str) -> String {
    if args.is_empty() {
        return "[]".to_owned();
    }
    if let (Some(first), Some(last)) = (args.first(), args.last()) {
        let first_span: oxc_span::Span = first.span();
        let last_span: oxc_span::Span = last.span();
        let first_start = usize::try_from(first_span.start).unwrap_or(0);
        let last_end = usize::try_from(last_span.end).unwrap_or(0);
        let args_text = source.get(first_start..last_end).unwrap_or("");
        format!("[{args_text}]")
    } else {
        "[]".to_owned()
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoArrayConstructor)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_new_array_multiple() {
        let diags = lint("var a = new Array(1, 2, 3);");
        assert_eq!(diags.len(), 1, "new Array(1, 2, 3) should be flagged");
    }

    #[test]
    fn test_flags_array_call_empty() {
        let diags = lint("var a = Array();");
        assert_eq!(diags.len(), 1, "Array() empty should be flagged");
    }

    #[test]
    fn test_allows_single_arg() {
        let diags = lint("var a = new Array(5);");
        assert!(
            diags.is_empty(),
            "new Array(5) creates sparse array — should not be flagged"
        );
    }

    #[test]
    fn test_allows_array_literal() {
        let diags = lint("var a = [1, 2, 3];");
        assert!(diags.is_empty(), "array literal should not be flagged");
    }
}
