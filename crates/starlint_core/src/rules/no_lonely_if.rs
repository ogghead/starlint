//! Rule: `no-lonely-if`
//!
//! Disallow `if` as the only statement in an `else` block.
//! `if (a) {} else { if (b) {} }` should be `if (a) {} else if (b) {}`.

use oxc_ast::AstKind;
use oxc_ast::ast::Statement;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `else { if (...) {} }` that should be `else if (...) {}`.
#[derive(Debug)]
pub struct NoLonelyIf;

impl NativeRule for NoLonelyIf {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-lonely-if".to_owned(),
            description: "Disallow `if` as the only statement in an `else` block".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::IfStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::IfStatement(stmt) = kind else {
            return;
        };

        // Check for `else { <single if statement> }`.
        let Some(Statement::BlockStatement(block)) = &stmt.alternate else {
            return;
        };

        if block.body.len() != 1 {
            return;
        }

        let Some(Statement::IfStatement(inner_if)) = block.body.first() else {
            return;
        };

        // Get the inner if-statement source text.
        let inner_start = usize::try_from(inner_if.span.start).unwrap_or(0);
        let inner_end = usize::try_from(inner_if.span.end).unwrap_or(0);
        let Some(inner_text) = ctx.source_text().get(inner_start..inner_end) else {
            return;
        };

        // Replace the block `{ if (...) {} }` with ` if (...) {}`.
        let replacement = format!(" {inner_text}");

        ctx.report(Diagnostic {
            rule_name: "no-lonely-if".to_owned(),
            message: "Unexpected lonely `if` inside `else` block".to_owned(),
            span: Span::new(block.span.start, block.span.end),
            severity: Severity::Warning,
            help: Some("Combine into `else if`".to_owned()),
            fix: Some(Fix {
                message: "Combine into `else if`".to_owned(),
                edits: vec![Edit {
                    span: Span::new(block.span.start, block.span.end),
                    replacement,
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
        let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) else {
            return vec![];
        };
        let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoLonelyIf)];
        traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
    }

    #[test]
    fn test_flags_lonely_if() {
        let diags = lint("if (a) {} else { if (b) {} }");
        assert_eq!(diags.len(), 1, "should flag lonely if in else block");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert!(fix.is_some(), "should provide a fix");
    }

    #[test]
    fn test_flags_lonely_if_with_else() {
        let diags = lint("if (a) {} else { if (b) {} else {} }");
        assert_eq!(
            diags.len(),
            1,
            "should flag even when inner if has its own else"
        );
    }

    #[test]
    fn test_ignores_direct_else_if() {
        let diags = lint("if (a) {} else if (b) {}");
        assert!(diags.is_empty(), "direct else-if should not be flagged");
    }

    #[test]
    fn test_ignores_multiple_statements_in_else() {
        let diags = lint("if (a) {} else { console.log(1); if (b) {} }");
        assert!(
            diags.is_empty(),
            "multiple statements in else should not be flagged"
        );
    }

    #[test]
    fn test_ignores_no_alternate() {
        let diags = lint("if (a) {}");
        assert!(diags.is_empty(), "if without else should not be flagged");
    }

    #[test]
    fn test_fix_replaces_block_with_inner_if() {
        let source = "if (a) {} else { if (b) { x(); } }";
        let diags = lint(source);
        assert_eq!(diags.len(), 1);
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        let replacement = fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str()));
        assert_eq!(
            replacement,
            Some(" if (b) { x(); }"),
            "fix should replace block with inner if"
        );
    }
}
