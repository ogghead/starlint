//! Rule: `typescript/explicit-function-return-type`
//!
//! Require explicit return types on functions and class methods. Functions
//! without explicit return types rely on type inference which can be fragile
//! and may lead to unexpected API contracts. Arrow functions that are
//! immediately assigned (e.g. callbacks) are excluded as this is a common
//! and acceptable pattern.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/explicit-function-return-type";

/// Flags functions and class methods that lack an explicit return type annotation.
#[derive(Debug)]
pub struct ExplicitFunctionReturnType;

impl NativeRule for ExplicitFunctionReturnType {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require explicit return types on functions and class methods".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::Function])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::Function(func) = kind else {
            return;
        };

        // Only check function declarations and expressions that have a body
        // (skip ambient declarations like `declare function ...`)
        if func.body.is_none() {
            return;
        }

        // If the function already has a return type, nothing to report
        if func.return_type.is_some() {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: RULE_NAME.to_owned(),
            message: "Missing return type on function — add an explicit return type annotation"
                .to_owned(),
            span: Span::new(func.span.start, func.span.end),
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

    /// Helper to lint source code as TypeScript.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ExplicitFunctionReturnType)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_function_without_return_type() {
        let diags = lint("function foo() { return 1; }");
        assert_eq!(
            diags.len(),
            1,
            "function without return type should be flagged"
        );
    }

    #[test]
    fn test_allows_function_with_return_type() {
        let diags = lint("function foo(): number { return 1; }");
        assert!(
            diags.is_empty(),
            "function with return type should not be flagged"
        );
    }

    #[test]
    fn test_flags_method_without_return_type() {
        let diags = lint("class Foo { bar() { return 1; } }");
        assert_eq!(
            diags.len(),
            1,
            "class method without return type should be flagged"
        );
    }

    #[test]
    fn test_allows_method_with_return_type() {
        let diags = lint("class Foo { bar(): number { return 1; } }");
        assert!(
            diags.is_empty(),
            "class method with return type should not be flagged"
        );
    }

    #[test]
    fn test_allows_declare_function() {
        let diags = lint("declare function foo(): void;");
        assert!(
            diags.is_empty(),
            "declare function should not be flagged (no body)"
        );
    }
}
