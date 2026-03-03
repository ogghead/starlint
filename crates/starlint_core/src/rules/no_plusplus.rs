//! Rule: `no-plusplus`
//!
//! Disallow the unary operators `++` and `--`. These can be confusing due
//! to automatic semicolon insertion and can be replaced with `+= 1`/`-= 1`.

use oxc_ast::AstKind;
use oxc_ast::ast::UpdateOperator;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `++` and `--` unary operators.
#[derive(Debug)]
pub struct NoPlusplus;

impl NativeRule for NoPlusplus {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-plusplus".to_owned(),
            description: "Disallow the unary operators `++` and `--`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::UpdateExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::UpdateExpression(update) = kind else {
            return;
        };

        let op_str = match update.operator {
            UpdateOperator::Increment => "++",
            UpdateOperator::Decrement => "--",
        };

        ctx.report_warning(
            "no-plusplus",
            &format!("Unary operator `{op_str}` used"),
            Span::new(update.span.start, update.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoPlusplus)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_increment() {
        let diags = lint("x++;");
        assert_eq!(diags.len(), 1, "++ should be flagged");
    }

    #[test]
    fn test_flags_decrement() {
        let diags = lint("x--;");
        assert_eq!(diags.len(), 1, "-- should be flagged");
    }

    #[test]
    fn test_flags_prefix_increment() {
        let diags = lint("++x;");
        assert_eq!(diags.len(), 1, "prefix ++ should be flagged");
    }

    #[test]
    fn test_allows_plus_equal() {
        let diags = lint("x += 1;");
        assert!(diags.is_empty(), "+= 1 should not be flagged");
    }
}
