//! Rule: `no-func-assign` (eslint)
//!
//! Disallow reassignment of function declarations. Reassigning a function
//! declaration is almost always a mistake.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;
use oxc_semantic::SymbolFlags;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags reassignment of function declarations.
#[derive(Debug)]
pub struct NoFuncAssign;

impl NativeRule for NoFuncAssign {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-func-assign".to_owned(),
            description: "Disallow reassignment of function declarations".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn needs_semantic(&self) -> bool {
        true
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::Function])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::Function(func) = kind else {
            return;
        };

        // Only check function declarations (not expressions)
        if !func.is_declaration() {
            return;
        }

        let Some(id) = &func.id else {
            return;
        };

        let Some(symbol_id) = id.symbol_id.get() else {
            return;
        };

        let Some(semantic) = ctx.semantic() else {
            return;
        };

        let scoping = semantic.scoping();

        // Check that this symbol has the Function flag
        let flags = scoping.symbol_flags(symbol_id);
        if !flags.contains(SymbolFlags::Function) {
            return;
        }

        // Check if any reference to this symbol is a write
        let has_write = scoping
            .get_resolved_references(symbol_id)
            .any(oxc_semantic::Reference::is_write);

        if has_write {
            ctx.report_error(
                "no-func-assign",
                &format!(
                    "'{}' is a function declaration and should not be reassigned",
                    id.name
                ),
                Span::new(id.span.start, id.span.end),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::{build_semantic, parse_file};
    use crate::traversal::traverse_and_lint_with_semantic;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let program = allocator.alloc(parsed.program);
            let semantic = build_semantic(program);
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoFuncAssign)];
            traverse_and_lint_with_semantic(
                program,
                &rules,
                source,
                Path::new("test.js"),
                Some(&semantic),
            )
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_function_reassignment() {
        let diags = lint("function foo() {} foo = bar;");
        assert_eq!(
            diags.len(),
            1,
            "reassigning function declaration should be flagged"
        );
    }

    #[test]
    fn test_allows_normal_function() {
        let diags = lint("function foo() {} foo();");
        assert!(diags.is_empty(), "calling function should not be flagged");
    }

    #[test]
    fn test_allows_function_expression() {
        let diags = lint("var foo = function() {}; foo = bar;");
        assert!(
            diags.is_empty(),
            "reassigning function expression should not be flagged"
        );
    }

    #[test]
    fn test_allows_different_name() {
        let diags = lint("function foo() {} bar = baz;");
        assert!(
            diags.is_empty(),
            "assigning different name should not be flagged"
        );
    }
}
