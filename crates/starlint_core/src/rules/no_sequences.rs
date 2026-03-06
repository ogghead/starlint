//! Rule: `no-sequences`
//!
//! Disallow comma operator usage. The comma operator is confusing and
//! error-prone — most uses are mistakes where a semicolon was intended.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags use of the comma (sequence) operator.
#[derive(Debug)]
pub struct NoSequences;

impl NativeRule for NoSequences {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-sequences".to_owned(),
            description: "Disallow comma operator".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::SequenceExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::SequenceExpression(seq) = kind else {
            return;
        };

        // Fix: replace sequence with the last expression (the value of a sequence)
        let fix = seq.expressions.last().map(|last_expr| {
            let last_span = last_expr.span();
            Fix {
                message: "Replace with the last expression".to_owned(),
                edits: vec![Edit {
                    span: Span::new(seq.span.start, seq.span.end),
                    replacement: ctx
                        .source_text()
                        .get(
                            usize::try_from(last_span.start).unwrap_or(0)
                                ..usize::try_from(last_span.end).unwrap_or(0),
                        )
                        .unwrap_or("")
                        .to_owned(),
                }],
            }
        });

        ctx.report(Diagnostic {
            rule_name: "no-sequences".to_owned(),
            message: "Unexpected use of comma operator".to_owned(),
            span: Span::new(seq.span.start, seq.span.end),
            severity: Severity::Warning,
            help: None,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoSequences)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_comma_operator() {
        let diags = lint("var x = (1, 2);");
        assert_eq!(diags.len(), 1, "comma operator should be flagged");
    }

    #[test]
    fn test_allows_comma_in_args() {
        let diags = lint("foo(1, 2);");
        assert!(
            diags.is_empty(),
            "comma in function arguments should not be flagged"
        );
    }

    #[test]
    fn test_allows_comma_in_array() {
        let diags = lint("var x = [1, 2];");
        assert!(diags.is_empty(), "comma in array should not be flagged");
    }

    #[test]
    fn test_allows_comma_in_var() {
        let diags = lint("var a = 1, b = 2;");
        assert!(
            diags.is_empty(),
            "comma in var declaration should not be flagged"
        );
    }
}
