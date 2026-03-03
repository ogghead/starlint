//! Rule: `no-this-assignment` (unicorn)
//!
//! Disallow assigning `this` to a variable. With arrow functions and
//! `.bind()`, there's no need for `var self = this`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `const self = this` and similar patterns.
#[derive(Debug)]
pub struct NoThisAssignment;

impl NativeRule for NoThisAssignment {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-this-assignment".to_owned(),
            description: "Disallow assigning `this` to a variable".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::VariableDeclarator])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::VariableDeclarator(decl) = kind else {
            return;
        };

        let Some(init) = &decl.init else {
            return;
        };

        if matches!(init, Expression::ThisExpression(_)) {
            ctx.report_warning(
                "no-this-assignment",
                "Do not assign `this` to a variable — use arrow functions or `.bind()` instead",
                Span::new(decl.span.start, decl.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoThisAssignment)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_this_assignment() {
        let diags = lint("const self = this;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_other_assignment() {
        let diags = lint("const x = 5;");
        assert!(diags.is_empty());
    }
}
