//! Rule: `no-array-reverse`
//!
//! Flag `.reverse()` which mutates the array in-place. Prefer the
//! non-mutating `.toReversed()` method instead.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.reverse()` calls which mutate the original array.
#[derive(Debug)]
pub struct NoArrayReverse;

impl NativeRule for NoArrayReverse {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-array-reverse".to_owned(),
            description: "Disallow `.reverse()` which mutates the array — prefer `.toReversed()`"
                .to_owned(),
            category: Category::Suggestion,
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

        if member.property.name.as_str() != "reverse" {
            return;
        }

        if !call.arguments.is_empty() {
            return;
        }

        // Fix: replace `.reverse` with `.toReversed` in the property name
        let fix = Some(Fix {
            message: "Replace `.reverse()` with `.toReversed()`".to_owned(),
            edits: vec![Edit {
                span: Span::new(member.property.span.start, member.property.span.end),
                replacement: "toReversed".to_owned(),
            }],
            is_snippet: false,
        });

        ctx.report(Diagnostic {
            rule_name: "no-array-reverse".to_owned(),
            message: "`.reverse()` mutates the array — consider `.toReversed()` instead".to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some("Replace `.reverse()` with `.toReversed()`".to_owned()),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoArrayReverse)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_reverse_no_args() {
        let diags = lint("arr.reverse();");
        assert_eq!(diags.len(), 1, ".reverse() should be flagged");
    }

    #[test]
    fn test_allows_to_reversed() {
        let diags = lint("arr.toReversed();");
        assert!(diags.is_empty(), ".toReversed() should not be flagged");
    }

    #[test]
    fn test_flags_str_reverse() {
        // Without type information we cannot distinguish str.reverse() from arr.reverse()
        let diags = lint("str.reverse();");
        assert_eq!(
            diags.len(),
            1,
            "str.reverse() should be flagged (no type info)"
        );
    }

    #[test]
    fn test_allows_reverse_with_args() {
        let diags = lint("arr.reverse(true);");
        assert!(
            diags.is_empty(),
            ".reverse() with arguments should not be flagged"
        );
    }
}
