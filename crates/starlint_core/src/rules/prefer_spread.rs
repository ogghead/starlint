//! Rule: `prefer-spread`
//!
//! Require spread operator instead of `.apply()`. `foo.apply(null, args)`
//! should be written as `foo(...args)`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.apply()` calls that could use spread syntax.
#[derive(Debug)]
pub struct PreferSpread;

impl NativeRule for PreferSpread {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-spread".to_owned(),
            description: "Require spread operator instead of `.apply()`".to_owned(),
            category: Category::Style,
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

        if member.property.name.as_str() != "apply" {
            return;
        }

        // Any .apply() call can potentially use spread
        ctx.report_warning(
            "prefer-spread",
            "Use the spread operator instead of `.apply()`",
            Span::new(call.span.start, call.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferSpread)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_apply() {
        let diags = lint("foo.apply(null, args);");
        assert_eq!(diags.len(), 1, ".apply() should be flagged");
    }

    #[test]
    fn test_allows_spread() {
        let diags = lint("foo(...args);");
        assert!(diags.is_empty(), "spread operator should not be flagged");
    }

    #[test]
    fn test_allows_normal_call() {
        let diags = lint("foo(1, 2);");
        assert!(diags.is_empty(), "normal call should not be flagged");
    }
}
