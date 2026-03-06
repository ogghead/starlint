//! Rule: `promise/prefer-await-to-callbacks`
//!
//! Prefer `async`/`await` over callback-style functions. Encourages
//! modern asynchronous patterns over Node.js-style error-first callbacks.

use oxc_ast::AstKind;
use oxc_ast::ast::BindingPattern;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Common callback parameter names that suggest callback-style code.
const CALLBACK_PARAMS: &[&str] = &["cb", "callback", "done", "next"];

/// Flags functions with callback-named parameters, suggesting `async`/`await`.
#[derive(Debug)]
pub struct PreferAwaitToCallbacks;

impl NativeRule for PreferAwaitToCallbacks {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/prefer-await-to-callbacks".to_owned(),
            description: "Prefer `async`/`await` over callbacks".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ArrowFunctionExpression, AstType::Function])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let params = match kind {
            AstKind::Function(func) => {
                if func.r#async {
                    return; // Already async, skip
                }
                &func.params
            }
            AstKind::ArrowFunctionExpression(arrow) => {
                if arrow.r#async {
                    return;
                }
                &arrow.params
            }
            _ => return,
        };

        for param in &params.items {
            if let BindingPattern::BindingIdentifier(id) = &param.pattern {
                let name = id.name.as_str();
                if CALLBACK_PARAMS.contains(&name) {
                    let span = match kind {
                        AstKind::Function(func) => Span::new(func.span.start, func.span.end),
                        AstKind::ArrowFunctionExpression(arrow) => {
                            Span::new(arrow.span.start, arrow.span.end)
                        }
                        _ => return,
                    };
                    ctx.report(Diagnostic {
                        rule_name: "promise/prefer-await-to-callbacks".to_owned(),
                        message: format!(
                            "Function has callback parameter `{name}` — prefer `async`/`await`"
                        ),
                        span,
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                    return; // Only report once per function
                }
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferAwaitToCallbacks)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_callback_param() {
        let diags = lint("function foo(callback) { callback(null, 1); }");
        assert_eq!(diags.len(), 1, "should flag function with callback param");
    }

    #[test]
    fn test_allows_async_function() {
        let diags = lint("async function foo(callback) { }");
        assert!(diags.is_empty(), "async function should not be flagged");
    }

    #[test]
    fn test_allows_normal_params() {
        let diags = lint("function foo(x, y) { return x + y; }");
        assert!(diags.is_empty(), "normal params should not be flagged");
    }
}
