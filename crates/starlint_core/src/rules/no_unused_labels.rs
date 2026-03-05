//! Rule: `no-unused-labels`
//!
//! Disallow unused labels. Labels that are not referenced by any `break` or
//! `continue` statement are likely mistakes and add unnecessary complexity.

use oxc_ast::AstKind;
use oxc_ast::ast::Statement;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags labeled statements where the label is never used by break/continue.
#[derive(Debug)]
pub struct NoUnusedLabels;

impl NativeRule for NoUnusedLabels {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unused-labels".to_owned(),
            description: "Disallow unused labels".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::LabeledStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::LabeledStatement(labeled) = kind else {
            return;
        };

        let label_name = labeled.label.name.as_str();

        // Check if the label is referenced in the body
        if !statement_references_label(&labeled.body, label_name) {
            let label_span = labeled.label.span();
            // Delete from the start of the labeled statement to the start of the body.
            let body_start = labeled.body.span().start;
            let delete_span = Span::new(labeled.span.start, body_start);

            ctx.report(Diagnostic {
                rule_name: "no-unused-labels".to_owned(),
                message: format!("Label `{label_name}` is defined but never used"),
                span: Span::new(label_span.start, label_span.end),
                severity: Severity::Error,
                help: Some(format!("Remove label `{label_name}`")),
                fix: Some(Fix {
                    message: format!("Remove label `{label_name}`"),
                    edits: vec![Edit {
                        span: delete_span,
                        replacement: String::new(),
                    }],
                }),
                labels: vec![],
            });
        }
    }
}

/// Check if a statement (or its children) contains a break/continue that
/// references the given label.
fn statement_references_label(stmt: &Statement<'_>, label: &str) -> bool {
    match stmt {
        Statement::BreakStatement(brk) => {
            brk.label.as_ref().is_some_and(|l| l.name.as_str() == label)
        }
        Statement::ContinueStatement(cont) => cont
            .label
            .as_ref()
            .is_some_and(|l| l.name.as_str() == label),
        Statement::BlockStatement(block) => block
            .body
            .iter()
            .any(|s| statement_references_label(s, label)),
        Statement::IfStatement(if_stmt) => {
            statement_references_label(&if_stmt.consequent, label)
                || if_stmt
                    .alternate
                    .as_ref()
                    .is_some_and(|alt| statement_references_label(alt, label))
        }
        Statement::WhileStatement(while_stmt) => {
            statement_references_label(&while_stmt.body, label)
        }
        Statement::DoWhileStatement(do_while) => statement_references_label(&do_while.body, label),
        Statement::ForStatement(for_stmt) => statement_references_label(&for_stmt.body, label),
        Statement::ForInStatement(for_in) => statement_references_label(&for_in.body, label),
        Statement::ForOfStatement(for_of) => statement_references_label(&for_of.body, label),
        Statement::SwitchStatement(switch) => switch.cases.iter().any(|case| {
            case.consequent
                .iter()
                .any(|s| statement_references_label(s, label))
        }),
        Statement::TryStatement(try_stmt) => {
            try_stmt
                .block
                .body
                .iter()
                .any(|s| statement_references_label(s, label))
                || try_stmt.handler.as_ref().is_some_and(|h| {
                    h.body
                        .body
                        .iter()
                        .any(|s| statement_references_label(s, label))
                })
                || try_stmt
                    .finalizer
                    .as_ref()
                    .is_some_and(|f| f.body.iter().any(|s| statement_references_label(s, label)))
        }
        Statement::LabeledStatement(labeled) => statement_references_label(&labeled.body, label),
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnusedLabels)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_unused_label() {
        let diags = lint("A: var foo = 0;");
        assert_eq!(diags.len(), 1, "unused label A should be flagged");
    }

    #[test]
    fn test_flags_unused_loop_label() {
        let diags = lint("B: for (var i = 0; i < 10; i++) { break; }");
        assert_eq!(
            diags.len(),
            1,
            "label B with unlabeled break should be flagged"
        );
    }

    #[test]
    fn test_allows_used_label_break() {
        let diags = lint("A: for (var i = 0; i < 10; i++) { break A; }");
        assert!(
            diags.is_empty(),
            "label A used in break should not be flagged"
        );
    }

    #[test]
    fn test_allows_used_label_continue() {
        let diags = lint("A: for (var i = 0; i < 10; i++) { continue A; }");
        assert!(
            diags.is_empty(),
            "label A used in continue should not be flagged"
        );
    }

    #[test]
    fn test_allows_nested_label_usage() {
        let diags = lint("A: for (;;) { for (;;) { break A; } }");
        assert!(
            diags.is_empty(),
            "label A used in nested break should not be flagged"
        );
    }
}
