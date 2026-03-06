//! Rule: `prefer-number-properties`
//!
//! Prefer `Number` static methods over global equivalents.
//! Flag `isNaN()`, `isFinite()`, `parseInt()`, `parseFloat()` as globals.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Global functions that should use `Number.*` equivalents.
const GLOBAL_FUNCTIONS: &[(&str, &str)] = &[
    ("isNaN", "Number.isNaN"),
    ("isFinite", "Number.isFinite"),
    ("parseInt", "Number.parseInt"),
    ("parseFloat", "Number.parseFloat"),
];

/// Flags global `isNaN`, `isFinite`, `parseInt`, `parseFloat` calls.
#[derive(Debug)]
pub struct PreferNumberProperties;

impl NativeRule for PreferNumberProperties {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-number-properties".to_owned(),
            description: "Prefer `Number` static methods over global equivalents".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let Expression::Identifier(id) = &call.callee else {
            return;
        };

        let name = id.name.as_str();
        let Some((_, replacement)) = GLOBAL_FUNCTIONS.iter().find(|(global, _)| *global == name)
        else {
            return;
        };

        ctx.report(Diagnostic {
            rule_name: "prefer-number-properties".to_owned(),
            message: format!("Use `{replacement}()` instead of `{name}()`"),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some(format!("Replace `{name}` with `{replacement}`")),
            fix: Some(Fix {
                message: format!("Replace `{name}` with `{replacement}`"),
                edits: vec![Edit {
                    span: Span::new(id.span.start, id.span.end),
                    replacement: (*replacement).to_owned(),
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

    fn lint(source: &str) -> Vec<Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferNumberProperties)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_global_is_nan() {
        let diags = lint("isNaN(x);");
        assert_eq!(diags.len(), 1, "should flag isNaN()");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("Number.isNaN"),
            "fix should replace with Number.isNaN"
        );
    }

    #[test]
    fn test_flags_global_parse_int() {
        let diags = lint("parseInt('10', 10);");
        assert_eq!(diags.len(), 1, "should flag parseInt()");
    }

    #[test]
    fn test_allows_number_is_nan() {
        let diags = lint("Number.isNaN(x);");
        assert!(diags.is_empty(), "Number.isNaN() should not be flagged");
    }

    #[test]
    fn test_allows_number_parse_int() {
        let diags = lint("Number.parseInt('10', 10);");
        assert!(diags.is_empty(), "Number.parseInt() should not be flagged");
    }
}
