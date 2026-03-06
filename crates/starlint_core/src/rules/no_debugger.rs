//! Rule: `no-debugger`
//!
//! Disallow `debugger` statements. These should never appear in production code.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::rule::{NativeLintContext, NativeRule};

/// Flags `debugger` statements and offers a safe fix to remove them.
#[derive(Debug)]
pub struct NoDebugger;

impl NativeRule for NoDebugger {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-debugger".to_owned(),
            description: "Disallow `debugger` statements".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::DebuggerStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        if let AstKind::DebuggerStatement(stmt) = kind {
            let span = Span::new(stmt.span.start, stmt.span.end);
            ctx.report(Diagnostic {
                rule_name: "no-debugger".to_owned(),
                message: "Unexpected `debugger` statement".to_owned(),
                span,
                severity: Severity::Error,
                help: Some("Remove the `debugger` statement before deploying".to_owned()),
                fix: FixBuilder::new("Remove `debugger` statement")
                    .delete(span)
                    .build(),
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

    #[test]
    fn test_flags_debugger_statement() {
        let allocator = Allocator::default();
        let source = "debugger;\nconst x = 1;";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDebugger)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 1, "should find one debugger statement");
            let first = diags.first();
            assert_eq!(
                first.map(|d| d.rule_name.as_str()),
                Some("no-debugger"),
                "rule name should match"
            );
            assert!(
                first.is_some_and(|d| d.fix.is_some()),
                "should provide a fix"
            );
        }
    }

    #[test]
    fn test_clean_file_no_diagnostics() {
        let allocator = Allocator::default();
        let source = "const x = 1;\nexport default x;";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDebugger)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert!(diags.is_empty(), "clean file should have no diagnostics");
        }
    }

    #[test]
    fn test_multiple_debugger_statements() {
        let allocator = Allocator::default();
        let source = "debugger;\nconst x = 1;\ndebugger;";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDebugger)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 2, "should find two debugger statements");
        }
    }
}
