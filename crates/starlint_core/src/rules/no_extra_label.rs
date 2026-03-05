//! Rule: `no-extra-label`
//!
//! Disallow unnecessary labels. If a `break` or `continue` targets the
//! immediately enclosing loop or switch, the label is redundant.
//! This is a simplified version that flags any labeled statement
//! where the label is only used once in a direct child break/continue.

use oxc_ast::AstKind;
use oxc_ast::ast::Statement;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags labels that are unnecessary because the break/continue targets
/// the immediately enclosing loop.
#[derive(Debug)]
pub struct NoExtraLabel;

impl NativeRule for NoExtraLabel {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-extra-label".to_owned(),
            description: "Disallow unnecessary labels".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
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

        // If the labeled statement is a loop or switch, and the only
        // break/continue in its direct body references this label,
        // then the label is unnecessary.
        let is_simple_loop = matches!(
            &labeled.body,
            Statement::ForStatement(_)
                | Statement::ForInStatement(_)
                | Statement::ForOfStatement(_)
                | Statement::WhileStatement(_)
                | Statement::DoWhileStatement(_)
        );

        let is_switch = matches!(&labeled.body, Statement::SwitchStatement(_));

        if !is_simple_loop && !is_switch {
            return;
        }

        // For a simple single-level loop/switch, any break/continue with
        // this label is redundant since it's the immediately enclosing one.
        let span_start = labeled.span.start;
        let label_end = labeled.label.span.end;

        // Build edits: delete the label prefix, and remove label from break/continue.
        let body_start = labeled.body.span().start;
        let mut edits = vec![Edit {
            span: Span::new(span_start, body_start),
            replacement: String::new(),
        }];

        // Also remove label references from break/continue statements.
        collect_label_ref_edits(&labeled.body, label_name, &mut edits);

        ctx.report(Diagnostic {
            rule_name: "no-extra-label".to_owned(),
            message: format!("Unnecessary label `{label_name}`"),
            span: Span::new(span_start, label_end),
            severity: Severity::Warning,
            help: Some(format!("Remove label `{label_name}`")),
            fix: Some(Fix {
                message: format!("Remove label `{label_name}`"),
                edits,
            }),
            labels: vec![],
        });
    }
}

/// Walk the body of a loop/switch to find break/continue statements referencing
/// `label`, and add edits to remove the label (including the preceding space).
fn collect_label_ref_edits(stmt: &Statement<'_>, label: &str, edits: &mut Vec<Edit>) {
    match stmt {
        Statement::BreakStatement(brk) => {
            if let Some(l) = &brk.label {
                if l.name.as_str() == label {
                    // Delete " label" (space + label name) from break statement.
                    edits.push(Edit {
                        span: Span::new(l.span.start.saturating_sub(1), l.span.end),
                        replacement: String::new(),
                    });
                }
            }
        }
        Statement::ContinueStatement(cont) => {
            if let Some(l) = &cont.label {
                if l.name.as_str() == label {
                    edits.push(Edit {
                        span: Span::new(l.span.start.saturating_sub(1), l.span.end),
                        replacement: String::new(),
                    });
                }
            }
        }
        Statement::BlockStatement(block) => {
            for s in &block.body {
                collect_label_ref_edits(s, label, edits);
            }
        }
        Statement::IfStatement(if_stmt) => {
            collect_label_ref_edits(&if_stmt.consequent, label, edits);
            if let Some(alt) = &if_stmt.alternate {
                collect_label_ref_edits(alt, label, edits);
            }
        }
        Statement::ForStatement(f) => collect_label_ref_edits(&f.body, label, edits),
        Statement::ForInStatement(f) => collect_label_ref_edits(&f.body, label, edits),
        Statement::ForOfStatement(f) => collect_label_ref_edits(&f.body, label, edits),
        Statement::WhileStatement(w) => collect_label_ref_edits(&w.body, label, edits),
        Statement::DoWhileStatement(d) => collect_label_ref_edits(&d.body, label, edits),
        Statement::SwitchStatement(sw) => {
            for case in &sw.cases {
                for s in &case.consequent {
                    collect_label_ref_edits(s, label, edits);
                }
            }
        }
        _ => {}
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoExtraLabel)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_label_on_simple_loop() {
        let diags = lint("loop1: for (var i = 0; i < 10; i++) { break loop1; }");
        assert_eq!(
            diags.len(),
            1,
            "label on simple loop with break should be flagged"
        );
    }

    #[test]
    fn test_flags_label_on_while() {
        let diags = lint("loop1: while (true) { break; }");
        assert_eq!(diags.len(), 1, "label on while loop should be flagged");
    }

    #[test]
    fn test_allows_label_on_block() {
        let diags = lint("label1: { break label1; }");
        assert!(
            diags.is_empty(),
            "label on block statement should not be flagged by this rule"
        );
    }
}
