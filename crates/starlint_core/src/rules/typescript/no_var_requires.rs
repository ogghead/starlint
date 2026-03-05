//! Rule: `typescript/no-var-requires`
//!
//! Disallow `require()` in variable declarations. In `TypeScript` projects,
//! `require()` calls bypass the type system. Prefer `import` declarations
//! which are statically analyzed and type-checked.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags variable declarations whose initializer is a `require()` call.
#[derive(Debug)]
pub struct NoVarRequires;

impl NativeRule for NoVarRequires {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-var-requires".to_owned(),
            description: "Disallow `require()` in variable declarations".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
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

        if is_require_call(init) {
            ctx.report(Diagnostic {
                rule_name: "typescript/no-var-requires".to_owned(),
                message: "Use `import` instead of `require()` in variable declarations".to_owned(),
                span: Span::new(decl.span.start, decl.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if an expression is a call to `require`.
fn is_require_call(expr: &Expression<'_>) -> bool {
    let Expression::CallExpression(call) = expr else {
        return false;
    };

    matches!(&call.callee, Expression::Identifier(ident) if ident.name.as_str() == "require")
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoVarRequires)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_const_require() {
        let diags = lint("const x = require(\"foo\");");
        assert_eq!(diags.len(), 1, "`const x = require(...)` should be flagged");
    }

    #[test]
    fn test_flags_let_require() {
        let diags = lint("let x = require(\"bar\");");
        assert_eq!(diags.len(), 1, "`let x = require(...)` should be flagged");
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
        let diags = lint("const x = foo();");
        assert!(
            diags.is_empty(),
            "non-`require` call in variable init should not be flagged"
        );
    }

    #[test]
    fn test_allows_variable_without_init() {
        let diags = lint("let x;");
        assert!(
            diags.is_empty(),
            "variable without initializer should not be flagged"
        );
    }
}
