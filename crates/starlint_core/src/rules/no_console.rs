//! Rule: `no-console`
//!
//! Disallow `console.*` calls. Useful for production code where logging
//! should use a structured logger instead.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags calls to `console.*` methods.
#[derive(Debug)]
pub struct NoConsole;

impl NativeRule for NoConsole {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-console".to_owned(),
            description: "Disallow `console.*` calls".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        if let AstKind::CallExpression(call) = kind {
            if let Expression::StaticMemberExpression(member) = &call.callee {
                if let Expression::Identifier(ident) = &member.object {
                    if ident.name == "console" {
                        ctx.report_warning(
                            "no-console",
                            &format!("Unexpected `console.{}` call", member.property.name),
                            Span::new(call.span.start, call.span.end),
                        );
                    }
                }
            }
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
