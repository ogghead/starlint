//! Rule: `no-eval`
//!
//! Disallow the use of `eval()`. `eval()` is dangerous because it executes
//! arbitrary code with the caller's privileges and can be exploited for
//! code injection attacks.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags calls to `eval()`.
#[derive(Debug)]
pub struct NoEval;

impl NativeRule for NoEval {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-eval".to_owned(),
            description: "Disallow the use of `eval()`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let is_eval = match &call.callee {
            Expression::Identifier(id) => id.name.as_str() == "eval",
            Expression::StaticMemberExpression(member) => {
                member.property.name.as_str() == "eval"
                    && matches!(
                        &member.object,
                        Expression::Identifier(id) if id.name.as_str() == "window"
                            || id.name.as_str() == "globalThis"
                    )
            }
            _ => false,
        };

        if is_eval {
            ctx.report_warning(
                "no-eval",
                "`eval()` is a security risk and can be harmful",
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoEval)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_eval() {
        let diags = lint("eval('code');");
        assert_eq!(diags.len(), 1, "eval() should be flagged");
    }

    #[test]
    fn test_flags_window_eval() {
        let diags = lint("window.eval('code');");
        assert_eq!(diags.len(), 1, "window.eval() should be flagged");
    }

    #[test]
    fn test_allows_non_eval() {
        let diags = lint("foo('code');");
        assert!(diags.is_empty(), "non-eval call should not be flagged");
    }

    #[test]
    fn test_allows_eval_as_property() {
        let diags = lint("obj.eval('code');");
        assert!(
            diags.is_empty(),
            "eval as property of non-global should not be flagged"
        );
    }
}
