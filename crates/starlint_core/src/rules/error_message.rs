//! Rule: `error-message`
//!
//! Require error constructors to be called with a message argument.
//! `throw new Error()` without a message makes debugging harder.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Error constructors that should always have a message argument.
const ERROR_CONSTRUCTORS: &[&str] = &[
    "Error",
    "TypeError",
    "RangeError",
    "ReferenceError",
    "SyntaxError",
    "URIError",
    "EvalError",
];

/// Flags `new Error()` (and variants) without a message argument.
#[derive(Debug)]
pub struct ErrorMessage;

impl NativeRule for ErrorMessage {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "error-message".to_owned(),
            description: "Require error constructors to have a message argument".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::NewExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::NewExpression(new_expr) = kind else {
            return;
        };

        let Expression::Identifier(id) = &new_expr.callee else {
            return;
        };

        let name = id.name.as_str();
        if !ERROR_CONSTRUCTORS.contains(&name) {
            return;
        }

        if !new_expr.arguments.is_empty() {
            return;
        }

        // Fix: insert a placeholder message string inside the parens
        // `new Error()` → `new Error('')`
        let fix = {
            let source = ctx.source_text();
            #[allow(clippy::as_conversions)]
            source
                .get(new_expr.span.start as usize..new_expr.span.end as usize)
                .and_then(|text| {
                    text.rfind(')').map(|paren_pos| {
                        let insert_pos = new_expr
                            .span
                            .start
                            .saturating_add(u32::try_from(paren_pos).unwrap_or(0));
                        Fix {
                            message: "Add empty message `''`".to_owned(),
                            edits: vec![Edit {
                                span: Span::new(insert_pos, insert_pos),
                                replacement: "''".to_owned(),
                            }],
                            is_snippet: false,
                        }
                    })
                })
        };

        ctx.report(Diagnostic {
            rule_name: "error-message".to_owned(),
            message: format!("`new {name}()` should have a message argument"),
            span: Span::new(new_expr.span.start, new_expr.span.end),
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ErrorMessage)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_error_no_message() {
        let diags = lint("throw new Error();");
        assert_eq!(diags.len(), 1, "should flag new Error() without message");
    }

    #[test]
    fn test_flags_type_error_no_message() {
        let diags = lint("throw new TypeError();");
        assert_eq!(
            diags.len(),
            1,
            "should flag new TypeError() without message"
        );
    }

    #[test]
    fn test_allows_error_with_message() {
        let diags = lint("throw new Error('something went wrong');");
        assert!(diags.is_empty(), "Error with message should not be flagged");
    }

    #[test]
    fn test_allows_non_error_constructor() {
        let diags = lint("new MyClass();");
        assert!(
            diags.is_empty(),
            "non-error constructor should not be flagged"
        );
    }
}
