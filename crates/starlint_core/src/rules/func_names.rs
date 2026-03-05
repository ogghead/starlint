//! Rule: `func-names`
//!
//! Require or disallow named function expressions. Named functions
//! produce better stack traces and are easier to identify in debugging.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags anonymous function expressions that lack a name.
#[derive(Debug)]
pub struct FuncNames;

impl NativeRule for FuncNames {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "func-names".to_owned(),
            description: "Require named function expressions".to_owned(),
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

        // Only check function expressions, not declarations
        // Function declarations always have a name (parser enforces this)
        if func.is_declaration() {
            return;
        }

        // Function expression without a name
        if func.id.is_none() {
            ctx.report(Diagnostic {
                rule_name: "func-names".to_owned(),
                message: "Unexpected unnamed function expression".to_owned(),
                span: Span::new(func.span.start, func.span.end),
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(FuncNames)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_anonymous_function_expression() {
        let diags = lint("var foo = function() {};");
        assert_eq!(
            diags.len(),
            1,
            "anonymous function expression should be flagged"
        );
    }

    #[test]
    fn test_allows_named_function_expression() {
        let diags = lint("var foo = function bar() {};");
        assert!(
            diags.is_empty(),
            "named function expression should not be flagged"
        );
    }

    #[test]
    fn test_allows_function_declaration() {
        let diags = lint("function foo() {}");
        assert!(
            diags.is_empty(),
            "function declaration should not be flagged"
        );
    }

    #[test]
    fn test_flags_anonymous_callback() {
        let diags = lint("arr.forEach(function() {});");
        assert_eq!(diags.len(), 1, "anonymous callback should be flagged");
    }

    #[test]
    fn test_allows_arrow_function() {
        // Arrow functions are inherently anonymous; this rule only targets `function` expressions
        let diags = lint("var foo = () => {};");
        assert!(
            diags.is_empty(),
            "arrow functions should not be flagged by func-names"
        );
    }
}
