//! Rule: `typescript/no-this-alias`
//!
//! Disallow aliasing `this`. With arrow functions and `.bind()`, there is
//! rarely a need to assign `this` to a variable. The exception is
//! `const self = this`, which is a widely accepted convention.

use oxc_ast::AstKind;
use oxc_ast::ast::{BindingPattern, Expression};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags variable declarations that alias `this` (except `const self = this`).
#[derive(Debug)]
pub struct NoThisAlias;

impl NativeRule for NoThisAlias {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-this-alias".to_owned(),
            description: "Disallow aliasing `this`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::VariableDeclarator(decl) = kind else {
            return;
        };

        // Must have an initializer that is `this`
        let Some(init) = &decl.init else {
            return;
        };

        if !matches!(init, Expression::ThisExpression(_)) {
            return;
        }

        // Allow `const self = this` as a common acceptable pattern
        if let Some(name) = binding_name(&decl.id) {
            if name == "self" {
                return;
            }
        }

        ctx.report_warning(
            "typescript/no-this-alias",
            "Do not alias `this` — use arrow functions or `.bind()` instead",
            Span::new(decl.span.start, decl.span.end),
        );
    }
}

/// Extract a simple identifier name from a binding pattern.
///
/// Returns `None` for destructuring patterns (object, array, assignment).
fn binding_name<'a>(pattern: &'a BindingPattern<'a>) -> Option<&'a str> {
    match pattern {
        BindingPattern::BindingIdentifier(ident) => Some(ident.name.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code as TypeScript.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoThisAlias)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_that_equals_this() {
        let diags = lint("const that = this;");
        assert_eq!(diags.len(), 1, "`const that = this` should be flagged");
    }

    #[test]
    fn test_flags_underscore_this() {
        let diags = lint("const _this = this;");
        assert_eq!(diags.len(), 1, "`const _this = this` should be flagged");
    }

    #[test]
    fn test_allows_self_equals_this() {
        let diags = lint("const self = this;");
        assert!(
            diags.is_empty(),
            "`const self = this` should not be flagged"
        );
    }

    #[test]
    fn test_allows_other_assignment() {
        let diags = lint("const x = foo;");
        assert!(
            diags.is_empty(),
            "non-this assignment should not be flagged"
        );
    }
}
