//! Rule: `prefer-code-point` (unicorn)
//!
//! Prefer `String#codePointAt()` over `String#charCodeAt()` and
//! `String.fromCodePoint()` over `String.fromCharCode()`.
//! Code points handle surrogate pairs correctly while char codes do not.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `charCodeAt` and `fromCharCode` usage.
#[derive(Debug)]
pub struct PreferCodePoint;

impl NativeRule for PreferCodePoint {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-code-point".to_owned(),
            description:
                "Prefer `codePointAt` over `charCodeAt` and `fromCodePoint` over `fromCharCode`"
                    .to_owned(),
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

        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        let method = member.property.name.as_str();
        let replacement = match method {
            "charCodeAt" => "codePointAt",
            "fromCharCode" => "fromCodePoint",
            _ => return,
        };

        let prop_span = Span::new(member.property.span.start, member.property.span.end);

        ctx.report(Diagnostic {
            rule_name: "prefer-code-point".to_owned(),
            message: format!("Prefer `{replacement}()` over `{method}()`"),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some(format!("Replace `{method}` with `{replacement}`")),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: format!("Replace `{method}` with `{replacement}`"),
                edits: vec![Edit {
                    span: prop_span,
                    replacement: replacement.to_owned(),
                }],
                is_snippet: false,
            }),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferCodePoint)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_char_code_at() {
        let diags = lint("str.charCodeAt(0);");
        assert_eq!(diags.len(), 1, "charCodeAt should be flagged");
    }

    #[test]
    fn test_flags_from_char_code() {
        let diags = lint("String.fromCharCode(65);");
        assert_eq!(diags.len(), 1, "fromCharCode should be flagged");
    }

    #[test]
    fn test_allows_code_point_at() {
        let diags = lint("str.codePointAt(0);");
        assert!(diags.is_empty(), "codePointAt should not be flagged");
    }

    #[test]
    fn test_allows_from_code_point() {
        let diags = lint("String.fromCodePoint(65);");
        assert!(diags.is_empty(), "fromCodePoint should not be flagged");
    }
}
