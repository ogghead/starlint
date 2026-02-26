//! Rule: `no-inner-declarations`
//!
//! Disallow variable or function declarations in nested blocks.
//! Prior to ES6, function declarations were only allowed in the program body
//! or a function body. While ES6 allows block-level functions in strict mode,
//! `var` declarations in blocks are still hoisted and can be confusing.
//!
//! Note: A full implementation requires parent-chain / scope analysis to
//! determine whether a declaration is truly "inner". This stub registers the
//! rule metadata so it can be configured, but defers detection to a future
//! infrastructure pass.

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::Severity;
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags function or `var` declarations inside nested blocks.
///
/// Requires parent-chain access for full detection; currently a stub.
#[derive(Debug)]
pub struct NoInnerDeclarations;

impl NativeRule for NoInnerDeclarations {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-inner-declarations".to_owned(),
            description: "Disallow variable or function declarations in nested blocks".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, _kind: &AstKind<'_>, _ctx: &mut NativeLintContext<'_>) {
        // Full implementation requires parent-chain / scope analysis
        // (infra: scope-analysis-context). Stub for now.
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code with the `NoInnerDeclarations` rule.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoInnerDeclarations)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_allows_top_level_function() {
        let diags = lint("function foo() {}");
        assert!(diags.is_empty(), "top-level function should not be flagged");
    }

    #[test]
    fn test_allows_top_level_var() {
        let diags = lint("var x = 1;");
        assert!(diags.is_empty(), "top-level var should not be flagged");
    }

    #[test]
    fn test_allows_let_in_block() {
        let diags = lint("if (true) { let x = 1; }");
        assert!(diags.is_empty(), "let in block should not be flagged");
    }

    #[test]
    fn test_allows_const_in_block() {
        let diags = lint("if (true) { const x = 1; }");
        assert!(diags.is_empty(), "const in block should not be flagged");
    }
}
