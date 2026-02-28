//! Rule: `node/no-process-exit`
//!
//! Disallow the use of `process.exit()`. Calling `process.exit()` terminates
//! the process immediately without allowing cleanup handlers to run. Prefer
//! setting the exit code (`process.exitCode = 1`) and letting the process
//! exit naturally, or throwing an error.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags calls to `process.exit()`.
#[derive(Debug)]
pub struct NoProcessExit;

impl NativeRule for NoProcessExit {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "node/no-process-exit".to_owned(),
            description: "Disallow the use of `process.exit()`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "exit" {
            return;
        }

        let is_process = matches!(
            &member.object,
            Expression::Identifier(id) if id.name.as_str() == "process"
        );

        if is_process {
            ctx.report_warning(
                "node/no-process-exit",
                "Do not use `process.exit()` — set `process.exitCode` and allow the process to exit naturally",
                Span::new(call.span.start, call.span.end),
            );
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoProcessExit)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_process_exit() {
        let diags = lint("process.exit(1);");
        assert_eq!(diags.len(), 1, "process.exit() should be flagged");
    }

    #[test]
    fn test_flags_process_exit_no_args() {
        let diags = lint("process.exit();");
        assert_eq!(
            diags.len(),
            1,
            "process.exit() without args should be flagged"
        );
    }

    #[test]
    fn test_allows_process_env() {
        let diags = lint("const e = process.env.NODE_ENV;");
        assert!(diags.is_empty(), "process.env should not be flagged");
    }

    #[test]
    fn test_allows_other_exit() {
        let diags = lint("app.exit();");
        assert!(diags.is_empty(), "non-process exit should not be flagged");
    }
}
