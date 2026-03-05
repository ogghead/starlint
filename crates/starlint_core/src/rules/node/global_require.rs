//! Rule: `node/global-require`
//!
//! Disallow `require()` calls outside of the top-level module scope.
//! Calling `require()` inside functions, conditionals, or other nested
//! scopes makes dependency loading non-deterministic and harder to
//! statically analyze.

use std::sync::RwLock;

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `require()` calls that are not at the top-level module scope.
///
/// Uses a depth counter to track whether the current node is nested
/// inside a function or arrow function expression.
#[derive(Debug)]
pub struct GlobalRequire {
    /// Current function nesting depth (0 = top-level).
    depth: RwLock<u32>,
}

impl GlobalRequire {
    /// Create a new `GlobalRequire` rule instance.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            depth: RwLock::new(0),
        }
    }
}

impl Default for GlobalRequire {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeRule for GlobalRequire {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "node/global-require".to_owned(),
            description: "Disallow `require()` calls outside of the top-level module scope"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[
            AstType::ArrowFunctionExpression,
            AstType::CallExpression,
            AstType::Function,
        ])
    }

    fn leave_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[
            AstType::ArrowFunctionExpression,
            AstType::CallExpression,
            AstType::Function,
        ])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                if let Ok(mut guard) = self.depth.write() {
                    *guard = guard.saturating_add(1);
                }
            }
            AstKind::CallExpression(call) => {
                let is_require = matches!(
                    &call.callee,
                    Expression::Identifier(id) if id.name.as_str() == "require"
                );

                if !is_require {
                    return;
                }

                let inside_function = self.depth.read().ok().is_some_and(|guard| *guard > 0);

                if inside_function {
                    ctx.report(Diagnostic {
                        rule_name: "node/global-require".to_owned(),
                        message: "Unexpected `require()` inside a function \u{2014} move to top-level scope".to_owned(),
                        span: Span::new(call.span.start, call.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            _ => {}
        }
    }

    fn leave(&self, kind: &AstKind<'_>, _ctx: &mut NativeLintContext<'_>) {
        if matches!(
            kind,
            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)
        ) {
            if let Ok(mut guard) = self.depth.write() {
                *guard = guard.saturating_sub(1);
            }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(GlobalRequire::new())];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_require_inside_function() {
        let diags = lint("function f() { require('x'); }");
        assert_eq!(diags.len(), 1, "require inside function should be flagged");
    }

    #[test]
    fn test_flags_require_inside_arrow() {
        let diags = lint("const f = () => { require('x'); };");
        assert_eq!(
            diags.len(),
            1,
            "require inside arrow function should be flagged"
        );
    }

    #[test]
    fn test_allows_top_level_require() {
        let diags = lint("require('x');");
        assert!(diags.is_empty(), "top-level require should not be flagged");
    }

    #[test]
    fn test_allows_top_level_const_require() {
        let diags = lint("const x = require('x');");
        assert!(
            diags.is_empty(),
            "top-level const require should not be flagged"
        );
    }

    #[test]
    fn test_flags_nested_function_require() {
        let diags = lint("function a() { function b() { require('x'); } }");
        assert_eq!(diags.len(), 1, "deeply nested require should be flagged");
    }
}
