//! Rule: `no-array-reduce`
//!
//! Disallow `Array#reduce()` and `Array#reduceRight()`. These methods
//! often produce hard-to-read code. Prefer `for...of` loops or other
//! array methods for better readability.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags calls to `.reduce()` and `.reduceRight()`.
#[derive(Debug)]
pub struct NoArrayReduce;

impl NativeRule for NoArrayReduce {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-array-reduce".to_owned(),
            description: "Disallow Array#reduce() and Array#reduceRight()".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        let method = member.property.name.as_str();
        if method != "reduce" && method != "reduceRight" {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "no-array-reduce".to_owned(),
            message: format!("Prefer `for...of` or other array methods over `.{method}()`"),
            span: Span::new(call.span.start, call.span.end),
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoArrayReduce)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_reduce() {
        let diags = lint("arr.reduce((acc, x) => acc + x, 0);");
        assert_eq!(diags.len(), 1, ".reduce() should be flagged");
    }

    #[test]
    fn test_flags_reduce_right() {
        let diags = lint("arr.reduceRight((acc, x) => acc + x, 0);");
        assert_eq!(diags.len(), 1, ".reduceRight() should be flagged");
    }

    #[test]
    fn test_allows_map() {
        let diags = lint("arr.map(x => x + 1);");
        assert!(diags.is_empty(), ".map() should not be flagged");
    }

    #[test]
    fn test_allows_filter() {
        let diags = lint("arr.filter(x => x > 0);");
        assert!(diags.is_empty(), ".filter() should not be flagged");
    }

    #[test]
    fn test_allows_for_each() {
        let diags = lint("arr.forEach(x => console.log(x));");
        assert!(diags.is_empty(), ".forEach() should not be flagged");
    }
}
