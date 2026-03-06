//! Rule: `no-ternary`
//!
//! Disallow ternary operators. Some teams prefer `if/else` statements
//! for readability.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags all ternary (conditional) expressions.
#[derive(Debug)]
pub struct NoTernary;

impl NativeRule for NoTernary {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-ternary".to_owned(),
            description: "Disallow ternary operators".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ConditionalExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ConditionalExpression(cond) = kind else {
            return;
        };

        ctx.report(Diagnostic {
            rule_name: "no-ternary".to_owned(),
            message: "Unexpected use of ternary operator".to_owned(),
            span: Span::new(cond.span.start, cond.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoTernary)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_ternary() {
        let diags = lint("var x = a ? b : c;");
        assert_eq!(diags.len(), 1, "ternary expression should be flagged");
    }

    #[test]
    fn test_allows_if_else() {
        let diags = lint("var x; if (a) { x = b; } else { x = c; }");
        assert!(diags.is_empty(), "if-else should not be flagged");
    }
}
