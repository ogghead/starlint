//! Rule: `prefer-module`
//!
//! Prefer ESM (`import`/`export`) over `CommonJS` (`require`/`module.exports`).
//! Flags `require()` calls with a string argument, `module.exports = ...`,
//! and `exports.foo = ...` assignments.

use oxc_ast::AstKind;
use oxc_ast::ast::{AssignmentTarget, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `CommonJS` patterns in favor of ESM `import`/`export`.
#[derive(Debug)]
pub struct PreferModule;

impl NativeRule for PreferModule {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-module".to_owned(),
            description: "Prefer ESM `import`/`export` over CommonJS `require`/`module.exports`"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::AssignmentExpression, AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::CallExpression(call) => check_require(call, ctx),
            AstKind::AssignmentExpression(assign) => check_exports_assign(assign, ctx),
            _ => {}
        }
    }
}

/// Check for `require('...')` calls with a string literal argument.
fn check_require(call: &oxc_ast::ast::CallExpression<'_>, ctx: &mut NativeLintContext<'_>) {
    let Expression::Identifier(callee_id) = &call.callee else {
        return;
    };

    if callee_id.name.as_str() != "require" {
        return;
    }

    // Only flag require() with a single string argument.
    let has_string_arg = call
        .arguments
        .first()
        .is_some_and(|arg| matches!(arg, oxc_ast::ast::Argument::StringLiteral(_)));

    if has_string_arg {
        ctx.report_warning(
            "prefer-module",
            "Use ESM `import` instead of `require()`",
            Span::new(call.span.start, call.span.end),
        );
    }
}

/// Check for `module.exports = ...` and `exports.foo = ...` assignments.
fn check_exports_assign(
    assign: &oxc_ast::ast::AssignmentExpression<'_>,
    ctx: &mut NativeLintContext<'_>,
) {
    let is_commonjs_export = match &assign.left {
        // `module.exports = ...`
        AssignmentTarget::StaticMemberExpression(member) => {
            is_module_exports_target(member) || is_exports_property_target(member)
        }
        _ => false,
    };

    if is_commonjs_export {
        ctx.report_warning(
            "prefer-module",
            "Use ESM `export` instead of `CommonJS` `module.exports` / `exports`",
            Span::new(assign.span.start, assign.span.end),
        );
    }
}

/// Check if a static member expression target is `module.exports`.
fn is_module_exports_target(member: &oxc_ast::ast::StaticMemberExpression<'_>) -> bool {
    member.property.name.as_str() == "exports"
        && matches!(
            &member.object,
            Expression::Identifier(id) if id.name.as_str() == "module"
        )
}

/// Check if a static member expression target is `exports.foo`.
fn is_exports_property_target(member: &oxc_ast::ast::StaticMemberExpression<'_>) -> bool {
    matches!(
        &member.object,
        Expression::Identifier(id) if id.name.as_str() == "exports"
    )
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferModule)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_require() {
        let diags = lint("const x = require('foo');");
        assert_eq!(diags.len(), 1, "require() should be flagged");
    }

    #[test]
    fn test_flags_module_exports() {
        let diags = lint("module.exports = {};");
        assert_eq!(diags.len(), 1, "module.exports should be flagged");
    }

    #[test]
    fn test_flags_exports_property() {
        let diags = lint("exports.foo = bar;");
        assert_eq!(diags.len(), 1, "exports.foo should be flagged");
    }

    #[test]
    fn test_allows_esm_import() {
        let diags = lint("import x from 'foo';");
        assert!(diags.is_empty(), "ESM import should not be flagged");
    }

    #[test]
    fn test_allows_esm_export() {
        let diags = lint("export default {};");
        assert!(diags.is_empty(), "ESM export should not be flagged");
    }

    #[test]
    fn test_allows_require_without_string_arg() {
        let diags = lint("require(variable);");
        assert!(
            diags.is_empty(),
            "require() with non-string argument should not be flagged"
        );
    }

    #[test]
    fn test_allows_unrelated_assignment() {
        let diags = lint("foo.bar = 1;");
        assert!(
            diags.is_empty(),
            "unrelated assignment should not be flagged"
        );
    }
}
