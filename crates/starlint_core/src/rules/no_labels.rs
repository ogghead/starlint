//! Rule: `no-labels`
//!
//! Disallow labeled statements. Labels are rarely needed and can make
//! code harder to understand.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags labeled statements.
#[derive(Debug)]
pub struct NoLabels;

impl NativeRule for NoLabels {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-labels".to_owned(),
            description: "Disallow labeled statements".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::LabeledStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::LabeledStatement(stmt) = kind else {
            return;
        };

        ctx.report_warning(
            "no-labels",
            "Unexpected labeled statement",
            Span::new(stmt.span.start, stmt.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoLabels)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_labeled_statement() {
        let diags = lint("outer: for (var i = 0; i < 10; i++) { break outer; }");
        assert_eq!(diags.len(), 1, "labeled statement should be flagged");
    }

    #[test]
    fn test_allows_unlabeled_loop() {
        let diags = lint("for (var i = 0; i < 10; i++) { break; }");
        assert!(diags.is_empty(), "unlabeled loop should not be flagged");
    }
}
