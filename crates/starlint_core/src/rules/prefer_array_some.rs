//! Rule: `prefer-array-some` (unicorn)
//!
//! Prefer `.some()` over `.find()` when only checking for existence.
//! Using `.some()` returns a boolean directly and is more semantically
//! correct when you don't need the found element.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.find()` used in boolean contexts.
#[derive(Debug)]
pub struct PreferArraySome;

impl NativeRule for PreferArraySome {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-array-some".to_owned(),
            description: "Prefer .some() over .find() for existence checks".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::IfStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        // Look for `if (arr.find(...))` — find used in boolean context
        let AstKind::IfStatement(if_stmt) = kind else {
            return;
        };

        if let Some(prop_span) = find_property_span(&if_stmt.test) {
            let span = if_stmt.test.span();
            ctx.report(Diagnostic {
                rule_name: "prefer-array-some".to_owned(),
                message: "Prefer `.some()` over `.find()` when checking for existence".to_owned(),
                span: Span::new(span.start, span.end),
                severity: Severity::Warning,
                help: Some("Replace `.find()` with `.some()`".to_owned()),
                fix: Some(Fix {
                    message: "Replace `.find()` with `.some()`".to_owned(),
                    edits: vec![Edit {
                        span: prop_span,
                        replacement: "some".to_owned(),
                    }],
                }),
                labels: vec![],
            });
        }
    }
}

/// Check if an expression is a `.find(...)` call and return the property name span.
fn find_property_span(expr: &Expression<'_>) -> Option<Span> {
    let Expression::CallExpression(call) = expr else {
        return None;
    };

    let Expression::StaticMemberExpression(member) = &call.callee else {
        return None;
    };

    (member.property.name == "find")
        .then(|| Span::new(member.property.span.start, member.property.span.end))
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferArraySome)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_find_in_if() {
        let diags = lint("if (arr.find(x => x > 0)) { }");
        assert_eq!(diags.len(), 1, "find in if condition should be flagged");
    }

    #[test]
    fn test_allows_some() {
        let diags = lint("if (arr.some(x => x > 0)) { }");
        assert!(diags.is_empty(), "some should not be flagged");
    }

    #[test]
    fn test_allows_find_in_assignment() {
        let diags = lint("var item = arr.find(x => x > 0);");
        assert!(diags.is_empty(), "find in assignment should not be flagged");
    }
}
