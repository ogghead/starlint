//! Rule: `no-process-exit` (unicorn)
//!
//! Disallow `process.exit()`. Prefer throwing an error or using
//! `process.exitCode` to allow cleanup and graceful shutdown.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `process.exit()` calls.
#[derive(Debug)]
pub struct NoProcessExit;

impl NativeRule for NoProcessExit {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-process-exit".to_owned(),
            description: "Disallow `process.exit()`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let is_process_exit = matches!(
            &call.callee,
            Expression::StaticMemberExpression(member)
                if member.property.name.as_str() == "exit"
                && matches!(&member.object, Expression::Identifier(id) if id.name.as_str() == "process")
        );

        if is_process_exit {
            // Fix: `process.exit(N)` → `process.exitCode = N`
            #[allow(clippy::as_conversions)]
            let fix = call.arguments.first().and_then(|arg| {
                let source = ctx.source_text();
                let arg_span = arg.span();
                let arg_text = source.get(arg_span.start as usize..arg_span.end as usize)?;
                Some(Fix {
                    message: format!("Replace with `process.exitCode = {arg_text}`"),
                    edits: vec![Edit {
                        span: Span::new(call.span.start, call.span.end),
                        replacement: format!("process.exitCode = {arg_text}"),
                    }],
                    is_snippet: false,
                })
            });

            ctx.report(Diagnostic {
                rule_name: "no-process-exit".to_owned(),
                message:
                    "Avoid `process.exit()` — use `process.exitCode` or throw an error instead"
                        .to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some("Set `process.exitCode` instead for graceful shutdown".to_owned()),
                fix,
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
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_process_exit_code() {
        let diags = lint("process.exitCode = 1;");
        assert!(diags.is_empty());
    }
}
