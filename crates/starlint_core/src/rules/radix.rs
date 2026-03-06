//! Rule: `radix`
//!
//! Require the radix parameter in `parseInt()`. Without specifying the radix,
//! `parseInt()` can produce unexpected results for strings with leading zeros.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `parseInt()` calls without a radix argument.
#[derive(Debug)]
pub struct Radix;

impl NativeRule for Radix {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "radix".to_owned(),
            description: "Require radix parameter in `parseInt()`".to_owned(),
            category: Category::Style,
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

        // Check for parseInt() or Number.parseInt()
        let is_parse_int = match &call.callee {
            Expression::Identifier(id) => id.name.as_str() == "parseInt",
            Expression::StaticMemberExpression(member) => {
                member.property.name.as_str() == "parseInt"
                    && matches!(&member.object, Expression::Identifier(id) if id.name.as_str() == "Number")
            }
            _ => false,
        };

        if !is_parse_int {
            return;
        }

        // Must have at least one argument but missing the radix (second arg)
        if !call.arguments.is_empty() && call.arguments.len() < 2 {
            // Insert `, 10` before closing paren
            ctx.report(Diagnostic {
                rule_name: "radix".to_owned(),
                message: "Missing radix parameter in `parseInt()` — specify 10 for decimal"
                    .to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some("Add radix parameter `10`".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: "Add radix parameter `10`".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(
                            call.span.end.saturating_sub(1),
                            call.span.end.saturating_sub(1),
                        ),
                        replacement: ", 10".to_owned(),
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(Radix)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_missing_radix() {
        let diags = lint("var n = parseInt('071');");
        assert_eq!(diags.len(), 1, "parseInt without radix should be flagged");
    }

    #[test]
    fn test_allows_with_radix() {
        let diags = lint("var n = parseInt('071', 10);");
        assert!(
            diags.is_empty(),
            "parseInt with radix should not be flagged"
        );
    }

    #[test]
    fn test_flags_number_parse_int() {
        let diags = lint("var n = Number.parseInt('071');");
        assert_eq!(
            diags.len(),
            1,
            "Number.parseInt without radix should be flagged"
        );
    }

    #[test]
    fn test_allows_non_parse_int() {
        let diags = lint("var n = parseFloat('3.14');");
        assert!(diags.is_empty(), "parseFloat should not be flagged");
    }
}
