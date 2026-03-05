//! Rule: `typescript/no-for-in-array`
//!
//! Disallow iterating over arrays with `for...in`. The `for...in` statement
//! iterates over enumerable property *names* (string keys), not values.
//! When used on an array, the loop variable receives string indices (`"0"`,
//! `"1"`, ...) and may also include inherited enumerable properties. Use
//! `for...of` to iterate over array values instead.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags all `for...in` statements as potentially incorrect for array iteration.
#[derive(Debug)]
pub struct NoForInArray;

impl NativeRule for NoForInArray {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-for-in-array".to_owned(),
            description: "Disallow iterating over arrays with `for...in`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ForInStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ForInStatement(stmt) = kind else {
            return;
        };

        ctx.report(Diagnostic {
            rule_name: "typescript/no-for-in-array".to_owned(),
            message: "`for...in` iterates over string keys, not values — use `for...of` instead"
                .to_owned(),
            span: Span::new(stmt.span.start, stmt.span.end),
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

    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoForInArray)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_for_in_with_array() {
        let diags = lint("const arr = [1, 2, 3]; for (const key in arr) { console.log(key); }");
        assert_eq!(diags.len(), 1, "for-in on array should be flagged");
    }

    #[test]
    fn test_flags_for_in_with_variable() {
        let diags = lint("for (const k in someVar) {}");
        assert_eq!(diags.len(), 1, "for-in should be flagged");
    }

    #[test]
    fn test_flags_for_in_with_let() {
        let diags = lint("for (let key in obj) { use(key); }");
        assert_eq!(diags.len(), 1, "for-in with let should be flagged");
    }

    #[test]
    fn test_allows_for_of() {
        let diags = lint("for (const val of arr) { console.log(val); }");
        assert!(diags.is_empty(), "for-of should not be flagged");
    }

    #[test]
    fn test_allows_regular_for() {
        let diags = lint("for (let i = 0; i < 10; i++) {}");
        assert!(diags.is_empty(), "regular for loop should not be flagged");
    }
}
