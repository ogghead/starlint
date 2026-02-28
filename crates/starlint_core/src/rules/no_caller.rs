//! Rule: `no-caller`
//!
//! Disallow use of `arguments.caller` and `arguments.callee`. These are
//! deprecated and forbidden in strict mode.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `arguments.caller` and `arguments.callee`.
#[derive(Debug)]
pub struct NoCaller;

impl NativeRule for NoCaller {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-caller".to_owned(),
            description: "Disallow use of `arguments.caller` and `arguments.callee`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::StaticMemberExpression(member) = kind else {
            return;
        };

        let prop = member.property.name.as_str();
        if prop != "caller" && prop != "callee" {
            return;
        }

        let Expression::Identifier(id) = &member.object else {
            return;
        };

        if id.name.as_str() == "arguments" {
            ctx.report_warning(
                "no-caller",
                &format!("Avoid using `arguments.{prop}` — it is deprecated"),
                Span::new(member.span.start, member.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoCaller)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_arguments_callee() {
        let diags = lint("function f() { return arguments.callee; }");
        assert_eq!(diags.len(), 1, "arguments.callee should be flagged");
    }

    #[test]
    fn test_flags_arguments_caller() {
        let diags = lint("function f() { return arguments.caller; }");
        assert_eq!(diags.len(), 1, "arguments.caller should be flagged");
    }

    #[test]
    fn test_allows_other_properties() {
        let diags = lint("function f() { return arguments.length; }");
        assert!(diags.is_empty(), "arguments.length should not be flagged");
    }
}
