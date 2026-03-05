//! Rule: `no-multi-assign`
//!
//! Disallow chained assignment expressions like `a = b = c = 5`.
//! Chained assignments are hard to read and can lead to unexpected behavior.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags chained assignment expressions.
#[derive(Debug)]
pub struct NoMultiAssign;

impl NativeRule for NoMultiAssign {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-multi-assign".to_owned(),
            description: "Disallow use of chained assignment expressions".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::AssignmentExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::AssignmentExpression(assign) = kind else {
            return;
        };

        // Check if the right side is also an assignment
        if matches!(&assign.right, Expression::AssignmentExpression(_)) {
            ctx.report(Diagnostic {
                rule_name: "no-multi-assign".to_owned(),
                message: "Unexpected chained assignment".to_owned(),
                span: Span::new(assign.span.start, assign.span.end),
                severity: Severity::Warning,
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoMultiAssign)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_chained_assignment() {
        let diags = lint("a = b = c = 5;");
        assert!(!diags.is_empty(), "chained assignment should be flagged");
    }

    #[test]
    fn test_allows_single_assignment() {
        let diags = lint("a = 5;");
        assert!(diags.is_empty(), "single assignment should not be flagged");
    }
}
