//! Rule: `no-console`
//!
//! Disallow `console.*` calls. Useful for production code where logging
//! should use a structured logger instead.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::rule::{NativeLintContext, NativeRule};

/// Flags `console.*` call statements and offers to remove them.
///
/// Matches `ExpressionStatement` (not `CallExpression`) so the fix can cleanly
/// remove the entire statement. Console calls embedded in other expressions
/// (e.g. `const x = console.log(1)`) are not detected — this is a known
/// limitation, similar to computed access and aliased console.
#[derive(Debug)]
pub struct NoConsole;

impl NativeRule for NoConsole {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-console".to_owned(),
            description: "Disallow `console.*` calls".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ExpressionStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ExpressionStatement(stmt) = kind else {
            return;
        };
        let Expression::CallExpression(call) = &stmt.expression else {
            return;
        };
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };
        let Expression::Identifier(ident) = &member.object else {
            return;
        };
        if ident.name != "console" {
            return;
        }

        let span = Span::new(stmt.span.start, stmt.span.end);
        let fix = FixBuilder::new(
            format!("Remove `console.{}()` statement", member.property.name),
            FixKind::SuggestionFix,
        )
        .edit(fix_utils::delete_statement(ctx.source_text(), span))
        .build();
        ctx.report(Diagnostic {
            rule_name: "no-console".to_owned(),
            message: format!("Unexpected `console.{}` call", member.property.name),
            span,
            severity: Severity::Warning,
            help: Some("Remove the `console` call or replace with a logger".to_owned()),
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

    #[test]
    fn test_flags_console_log() {
        let allocator = Allocator::default();
        let source = "console.log('hello');";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConsole)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 1, "should flag console.log");
            assert!(
                diags
                    .first()
                    .is_some_and(|d| d.message.contains("console.log")),
                "message should mention console.log"
            );
        }
    }

    #[test]
    fn test_flags_console_error() {
        let allocator = Allocator::default();
        let source = "console.error('fail');";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConsole)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 1, "should flag console.error");
        }
    }

    #[test]
    fn test_fix_removes_statement() {
        let allocator = Allocator::default();
        let source = "console.log('hello');";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConsole)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            let fix = diags.first().and_then(|d| d.fix.as_ref());
            assert!(fix.is_some(), "should provide a fix");
            let edit = fix.and_then(|f| f.edits.first());
            assert_eq!(
                edit.map(|e| e.replacement.as_str()),
                Some(""),
                "fix should remove the statement"
            );
        }
    }

    #[test]
    fn test_ignores_embedded_console_call() {
        // Known limitation: console calls embedded in expressions are not detected
        // because we match ExpressionStatement (needed for clean statement removal).
        let allocator = Allocator::default();
        let source = "const x = console.log(1);";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConsole)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert!(
                diags.is_empty(),
                "embedded console call is a known false negative"
            );
        }
    }

    #[test]
    fn test_ignores_non_console() {
        let allocator = Allocator::default();
        let source = "logger.log('hello');";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConsole)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert!(diags.is_empty(), "should not flag logger.log");
        }
    }

    #[test]
    fn test_ignores_computed_console_access() {
        // Known limitation: computed member access like console["log"]() is not detected.
        let allocator = Allocator::default();
        let source = r#"console["log"]("hello");"#;
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConsole)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert!(
                diags.is_empty(),
                "computed console access is a known false negative"
            );
        }
    }

    #[test]
    fn test_ignores_aliased_console() {
        // Known limitation: `const c = console; c.log()` is not detected.
        let allocator = Allocator::default();
        let source = "const c = console; c.log('hello');";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConsole)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert!(
                diags.is_empty(),
                "aliased console access is a known false negative"
            );
        }
    }
}
