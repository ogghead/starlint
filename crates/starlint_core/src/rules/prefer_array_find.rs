//! Rule: `prefer-array-find` (unicorn)
//!
//! Prefer `.find()` over `.filter()[0]`. When only the first matching
//! element is needed, `.find()` is more efficient and readable.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.filter(...)[0]` patterns that should use `.find()`.
#[derive(Debug)]
pub struct PreferArrayFind;

impl NativeRule for PreferArrayFind {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-array-find".to_owned(),
            description: "Prefer .find() over .filter()[0]".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ComputedMemberExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ComputedMemberExpression(computed) = kind else {
            return;
        };

        // Check if the index is `0`
        let Expression::NumericLiteral(num) = &computed.expression else {
            return;
        };

        if num.value != 0.0 {
            return;
        }

        // Check if the object is a `.filter(...)` call
        let Expression::CallExpression(call) = &computed.object else {
            return;
        };

        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name == "filter" {
            // Two edits: rename .filter → .find, and delete [0]
            let prop_span = Span::new(member.property.span.start, member.property.span.end);
            // Delete from end of call expression to end of computed member (the `[0]`)
            let call_end = call.span.end;
            let computed_end = computed.span.end;

            ctx.report(Diagnostic {
                rule_name: "prefer-array-find".to_owned(),
                message: "Prefer `.find()` over `.filter()[0]`".to_owned(),
                span: Span::new(computed.span.start, computed.span.end),
                severity: Severity::Warning,
                help: Some("Replace `.filter()[0]` with `.find()`".to_owned()),
                fix: Some(Fix {
                    message: "Replace `.filter()[0]` with `.find()`".to_owned(),
                    edits: vec![
                        Edit {
                            span: prop_span,
                            replacement: "find".to_owned(),
                        },
                        Edit {
                            span: Span::new(call_end, computed_end),
                            replacement: String::new(),
                        },
                    ],
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferArrayFind)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_filter_zero() {
        let diags = lint("var x = arr.filter(fn)[0];");
        assert_eq!(diags.len(), 1, ".filter()[0] should be flagged");
    }

    #[test]
    fn test_allows_find() {
        let diags = lint("var x = arr.find(fn);");
        assert!(diags.is_empty(), ".find() should not be flagged");
    }

    #[test]
    fn test_allows_filter_non_zero() {
        let diags = lint("var x = arr.filter(fn)[1];");
        assert!(diags.is_empty(), ".filter()[1] should not be flagged");
    }

    #[test]
    fn test_allows_filter_variable() {
        let diags = lint("var x = arr.filter(fn);");
        assert!(
            diags.is_empty(),
            ".filter() without index should not be flagged"
        );
    }
}
