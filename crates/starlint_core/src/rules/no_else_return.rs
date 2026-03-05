//! Rule: `no-else-return`
//!
//! Disallow `else` blocks after `return` in `if` statements. If the `if`
//! block always returns, the `else` is unnecessary and the code can be
//! flattened.

use oxc_ast::AstKind;
use oxc_ast::ast::Statement;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags unnecessary `else` blocks after `return`.
#[derive(Debug)]
pub struct NoElseReturn;

impl NativeRule for NoElseReturn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-else-return".to_owned(),
            description: "Disallow `else` blocks after `return` in `if` statements".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::IfStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::IfStatement(if_stmt) = kind else {
            return;
        };

        // Must have an else block
        if if_stmt.alternate.is_none() {
            return;
        }

        // The consequent must always return
        if !consequent_always_returns(&if_stmt.consequent) {
            return;
        }

        let source = ctx.source_text();
        let Some(alternate) = if_stmt.alternate.as_ref() else {
            return;
        };
        let alt_start = usize::try_from(alternate.span().start).unwrap_or(0);
        let alt_end = usize::try_from(alternate.span().end).unwrap_or(0);
        let alt_source = source.get(alt_start..alt_end).unwrap_or("");

        // If alternate is a block, extract inner content (strip braces)
        let body_text = if matches!(alternate, Statement::BlockStatement(_)) {
            alt_source
                .get(1..alt_source.len().saturating_sub(1))
                .unwrap_or("")
                .trim()
        } else {
            alt_source.trim()
        };

        // Replace ` else { ... }` (from consequent end to if_stmt end)
        // with `\n` + the body statements
        let cons_end = if_stmt.consequent.span().end;

        ctx.report(Diagnostic {
            rule_name: "no-else-return".to_owned(),
            message:
                "Unnecessary `else` after `return` — remove the `else` and outdent its contents"
                    .to_owned(),
            span: Span::new(if_stmt.span.start, if_stmt.span.end),
            severity: Severity::Warning,
            help: Some("Remove the `else` wrapper".to_owned()),
            fix: Some(Fix {
                message: "Remove the `else` wrapper".to_owned(),
                edits: vec![Edit {
                    span: Span::new(cons_end, if_stmt.span.end),
                    replacement: format!("\n{body_text}"),
                }],
            }),
            labels: vec![],
        });
    }
}

/// Check if a statement always returns.
fn consequent_always_returns(stmt: &Statement<'_>) -> bool {
    match stmt {
        Statement::ReturnStatement(_) => true,
        Statement::BlockStatement(block) => block
            .body
            .last()
            .is_some_and(|last| matches!(last, Statement::ReturnStatement(_))),
        _ => false,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoElseReturn)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_else_after_return() {
        let diags = lint("function f(x) { if (x) { return 1; } else { return 2; } }");
        assert_eq!(diags.len(), 1, "else after return should be flagged");
    }

    #[test]
    fn test_allows_no_else() {
        let diags = lint("function f(x) { if (x) { return 1; } return 2; }");
        assert!(diags.is_empty(), "no else should not be flagged");
    }

    #[test]
    fn test_allows_no_return_in_if() {
        let diags = lint("function f(x) { if (x) { foo(); } else { bar(); } }");
        assert!(diags.is_empty(), "if without return should not be flagged");
    }
}
