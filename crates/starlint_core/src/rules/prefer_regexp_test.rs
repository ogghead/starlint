//! Rule: `prefer-regexp-test` (unicorn)
//!
//! Prefer `RegExp#test()` over `String#match()` when only checking for
//! existence of a match. `test()` is faster and more semantically correct
//! when you don't need the matched value.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `string.match(regex)` that could use `regex.test(string)`.
#[derive(Debug)]
pub struct PreferRegexpTest;

impl NativeRule for PreferRegexpTest {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-regexp-test".to_owned(),
            description: "Prefer RegExp#test() over String#match()".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        // Look for `something.match(arg)` used in a boolean context
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check if callee is `something.match`
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name != "match" {
            return;
        }

        // Must have exactly one argument
        if call.arguments.len() != 1 {
            return;
        }

        // Check if the argument is a regex literal
        let Some(arg) = call.arguments.first() else {
            return;
        };
        let is_regex_arg = matches!(arg, oxc_ast::ast::Argument::RegExpLiteral(_));

        if is_regex_arg {
            let source = ctx.source_text();
            let obj_span = member.object.span();
            let str_text = source[obj_span.start as usize..obj_span.end as usize].to_owned();
            let regex_span = arg.span();
            let regex_text = source[regex_span.start as usize..regex_span.end as usize].to_owned();
            let replacement = format!("{regex_text}.test({str_text})");

            ctx.report(Diagnostic {
                rule_name: "prefer-regexp-test".to_owned(),
                message: "Prefer `RegExp#test()` over `String#match()`".to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some(format!("Replace with `{replacement}`")),
                fix: Some(Fix {
                    message: format!("Replace with `{replacement}`"),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferRegexpTest)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_match_with_regex() {
        let diags = lint("if (str.match(/foo/)) {}");
        assert_eq!(diags.len(), 1, "match with regex literal should be flagged");
    }

    #[test]
    fn test_allows_match_with_string() {
        let diags = lint("str.match('foo');");
        assert!(
            diags.is_empty(),
            "match with string argument should not be flagged"
        );
    }

    #[test]
    fn test_allows_test() {
        let diags = lint("/foo/.test(str);");
        assert!(diags.is_empty(), "test() should not be flagged");
    }

    #[test]
    fn test_allows_match_multiple_args() {
        let diags = lint("str.match(/foo/, 'g');");
        assert!(
            diags.is_empty(),
            "match with multiple args should not be flagged"
        );
    }
}
