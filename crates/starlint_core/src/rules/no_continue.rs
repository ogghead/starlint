//! Rule: `no-continue`
//!
//! Disallow `continue` statements. Some style guides forbid `continue`
//! because it can make control flow harder to follow.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `continue` statements.
#[derive(Debug)]
pub struct NoContinue;

impl NativeRule for NoContinue {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-continue".to_owned(),
            description: "Disallow `continue` statements".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ContinueStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ContinueStatement(stmt) = kind else {
            return;
        };

        ctx.report(Diagnostic {
            rule_name: "no-continue".to_owned(),
            message: "Unexpected use of `continue` statement".to_owned(),
            span: Span::new(stmt.span.start, stmt.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoContinue)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_continue() {
        let diags = lint("for (var i = 0; i < 10; i++) { if (i === 5) continue; }");
        assert_eq!(diags.len(), 1, "continue should be flagged");
    }

    #[test]
    fn test_allows_loop_without_continue() {
        let diags = lint("for (var i = 0; i < 10; i++) { foo(i); }");
        assert!(
            diags.is_empty(),
            "loop without continue should not be flagged"
        );
    }
}
