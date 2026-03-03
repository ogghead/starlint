//! Rule: `typescript/no-require-imports`
//!
//! Disallow `require()` calls entirely. In `TypeScript` projects, `require()`
//! bypasses the module type system. Use `import` declarations instead, which
//! are statically analyzed and provide better tooling support.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags any `require()` call expression.
#[derive(Debug)]
pub struct NoRequireImports;

impl NativeRule for NoRequireImports {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-require-imports".to_owned(),
            description: "Disallow `require()` calls".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let Expression::Identifier(ident) = &call.callee else {
            return;
        };

        if ident.name.as_str() != "require" {
            return;
        }

        ctx.report_warning(
            "typescript/no-require-imports",
            "Use `import` instead of `require()`",
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

    /// Helper to lint source code as TypeScript.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoRequireImports)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_bare_require() {
        let diags = lint("require(\"foo\");");
        assert_eq!(diags.len(), 1, "bare `require()` call should be flagged");
    }

    #[test]
    fn test_flags_require_in_variable() {
        let diags = lint("const x = require(\"bar\");");
        assert_eq!(
            diags.len(),
            1,
            "`require()` in variable init should be flagged"
        );
    }

    #[test]
    fn test_allows_import() {
        let diags = lint("import x from \"foo\";");
        assert!(
            diags.is_empty(),
            "`import` declaration should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_require_call() {
        let diags = lint("foo();");
        assert!(diags.is_empty(), "non-`require` call should not be flagged");
    }

    #[test]
    fn test_allows_method_named_require() {
        let diags = lint("obj.require(\"foo\");");
        assert!(
            diags.is_empty(),
            "method call named `require` should not be flagged"
        );
    }
}
