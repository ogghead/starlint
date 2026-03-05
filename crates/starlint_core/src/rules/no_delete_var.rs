//! Rule: `no-delete-var`
//!
//! Disallow deleting variables. The `delete` operator is meant for removing
//! properties from objects. Using `delete` on a variable is either a mistake
//! or produces confusing, implementation-dependent behavior.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, UnaryOperator};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `delete` applied directly to a variable identifier.
#[derive(Debug)]
pub struct NoDeleteVar;

impl NativeRule for NoDeleteVar {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-delete-var".to_owned(),
            description: "Disallow deleting variables".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::UnaryExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::UnaryExpression(expr) = kind else {
            return;
        };

        if expr.operator != UnaryOperator::Delete {
            return;
        }

        // Only flag when the operand is a plain identifier (i.e. a variable).
        // `delete obj.prop` is fine — that's the intended usage.
        if matches!(&expr.argument, Expression::Identifier(_)) {
            ctx.report(Diagnostic {
                rule_name: "no-delete-var".to_owned(),
                message: "Variables should not be deleted".to_owned(),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDeleteVar)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_delete_variable() {
        let diags = lint("var x = 1; delete x;");
        assert_eq!(diags.len(), 1, "delete x should be flagged");
    }

    #[test]
    fn test_allows_delete_property() {
        let diags = lint("delete obj.prop;");
        assert!(diags.is_empty(), "delete obj.prop should not be flagged");
    }

    #[test]
    fn test_allows_delete_computed_property() {
        let diags = lint("delete obj['key'];");
        assert!(diags.is_empty(), "delete obj['key'] should not be flagged");
    }

    #[test]
    fn test_allows_non_delete_unary() {
        let diags = lint("typeof x;");
        assert!(diags.is_empty(), "typeof should not be flagged");
    }
}
