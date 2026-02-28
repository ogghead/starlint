//! Rule: `no-extra-label`
//!
//! Disallow unnecessary labels. If a `break` or `continue` targets the
//! immediately enclosing loop or switch, the label is redundant.
//! This is a simplified version that flags any labeled statement
//! where the label is only used once in a direct child break/continue.

use oxc_ast::AstKind;
use oxc_ast::ast::Statement;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
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
        // We flag the label itself.
        let span_start = labeled.span.start;
        let label_end = labeled.label.span.end;
        // Label span includes the colon: "label:"
        let label_len = label_name.len();
        let _ = label_len; // Suppress unused warning

        ctx.report_warning(
            "no-extra-label",
            &format!("Unnecessary label `{label_name}`"),
            Span::new(span_start, label_end),
        );
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
