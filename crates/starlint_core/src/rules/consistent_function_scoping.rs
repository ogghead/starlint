//! Rule: `consistent-function-scoping`
//!
//! Flag functions that don't need to be nested inside another function.
//! Named function declarations inside other functions can usually be moved
//! to the top level or a higher scope, improving readability and testability.
//!
//! This is a simplified version that flags named function declarations
//! (not expressions or arrow functions) that appear inside other function
//! bodies.

use std::sync::RwLock;

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags named function declarations nested inside other functions.
#[derive(Debug)]
pub struct ConsistentFunctionScoping {
    /// Current function nesting depth during traversal.
    depth: RwLock<u32>,
}

impl ConsistentFunctionScoping {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            depth: RwLock::new(0),
        }
    }
}

impl Default for ConsistentFunctionScoping {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeRule for ConsistentFunctionScoping {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "consistent-function-scoping".to_owned(),
            description: "Flag functions that could be moved to a higher scope".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::Function(f) => {
                let Ok(mut depth) = self.depth.write() else {
                    return;
                };

                // Only flag named function declarations (not expressions)
                if f.is_declaration() && *depth > 0 {
                    let name = f.id.as_ref().map_or("anonymous", |id| id.name.as_str());
                    ctx.report_warning(
                        "consistent-function-scoping",
                        &format!(
                            "Function `{name}` is declared inside another function and could be moved to a higher scope"
                        ),
                        Span::new(f.span.start, f.span.end),
                    );
                }

                // Track depth for all functions (declarations and expressions)
                *depth = depth.saturating_add(1);
            }
            AstKind::ArrowFunctionExpression(_) => {
                let Ok(mut depth) = self.depth.write() else {
                    return;
                };
                *depth = depth.saturating_add(1);
            }
            _ => {}
        }
    }

    fn leave(&self, kind: &AstKind<'_>, _ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                if let Ok(mut depth) = self.depth.write() {
                    *depth = depth.saturating_sub(1);
                }
            }
            _ => {}
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ConsistentFunctionScoping::new())];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_nested_named_function() {
        let diags = lint("function outer() { function inner() { return 1; } return inner(); }");
        assert_eq!(diags.len(), 1, "nested named function should be flagged");
    }

    #[test]
    fn test_allows_anonymous_function_expression() {
        let diags = lint("function outer() { return function() { return 1; }; }");
        assert!(
            diags.is_empty(),
            "anonymous function expression should not be flagged"
        );
    }

    #[test]
    fn test_allows_top_level_function() {
        let diags = lint("function foo() { return 1; }");
        assert!(diags.is_empty(), "top-level function should not be flagged");
    }

    #[test]
    fn test_allows_arrow_inside_function() {
        let diags = lint("function outer() { const inner = () => 1; return inner(); }");
        assert!(
            diags.is_empty(),
            "arrow function inside function should not be flagged"
        );
    }

    #[test]
    fn test_flags_named_function_inside_arrow() {
        let diags = lint("const outer = () => { function inner() { return 1; } return inner(); };");
        assert_eq!(
            diags.len(),
            1,
            "named function inside arrow should be flagged"
        );
    }

    #[test]
    fn test_allows_multiple_top_level_functions() {
        let diags = lint("function foo() { return 1; } function bar() { return 2; }");
        assert!(
            diags.is_empty(),
            "multiple top-level functions should not be flagged"
        );
    }

    #[test]
    fn test_flags_deeply_nested() {
        let diags = lint(
            "function a() { function b() { function c() { return 1; } return c(); } return b(); }",
        );
        assert_eq!(
            diags.len(),
            2,
            "both b (inside a) and c (inside b) should be flagged"
        );
    }
}
