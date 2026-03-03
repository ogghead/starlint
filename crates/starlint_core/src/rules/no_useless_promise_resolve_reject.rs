//! Rule: `no-useless-promise-resolve-reject` (unicorn)
//!
//! Disallow wrapping values in `Promise.resolve()` or `Promise.reject()`
//! unnecessarily within async functions, where you can simply return/throw
//! the value directly.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags unnecessary `Promise.resolve()`/`Promise.reject()` in async functions.
#[derive(Debug)]
pub struct NoUselessPromiseResolveReject;

impl NativeRule for NoUselessPromiseResolveReject {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-promise-resolve-reject".to_owned(),
            description: "Disallow unnecessary Promise.resolve/reject in async functions"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn needs_semantic(&self) -> bool {
        true
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[
            AstType::ArrowFunctionExpression,
            AstType::Function,
            AstType::ReturnStatement,
        ])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        // Look for return statements
        let AstKind::ReturnStatement(ret) = kind else {
            return;
        };

        let Some(arg) = &ret.argument else {
            return;
        };

        // Check if the return value is Promise.resolve(...) or Promise.reject(...)
        let Some(method_name) = is_promise_resolve_or_reject(arg) else {
            return;
        };

        // Walk ancestors to check if we're inside an async function
        let Some(semantic) = ctx.semantic() else {
            return;
        };

        let Some(node_id) = find_node_id_by_span(semantic, ret.span) else {
            return;
        };

        // Walk ancestors to find the nearest function
        for ancestor in semantic.nodes().ancestors(node_id) {
            match ancestor.kind() {
                AstKind::Function(func) if func.r#async => {
                    ctx.report_warning(
                        "no-useless-promise-resolve-reject",
                        &format!(
                            "Unnecessary `Promise.{method_name}()` in async function; \
                             use `return` or `throw` directly"
                        ),
                        Span::new(ret.span.start, ret.span.end),
                    );
                    return;
                }
                AstKind::ArrowFunctionExpression(arrow) if arrow.r#async => {
                    ctx.report_warning(
                        "no-useless-promise-resolve-reject",
                        &format!(
                            "Unnecessary `Promise.{method_name}()` in async function; \
                             use `return` or `throw` directly"
                        ),
                        Span::new(ret.span.start, ret.span.end),
                    );
                    return;
                }
                // Hit a non-async function boundary, stop
                AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                    return;
                }
                _ => {}
            }
        }
    }
}

/// Check if an expression is `Promise.resolve(...)` or `Promise.reject(...)`.
/// Returns the method name if it matches.
fn is_promise_resolve_or_reject<'a>(expr: &'a Expression<'_>) -> Option<&'a str> {
    let Expression::CallExpression(call) = expr else {
        return None;
    };

    let Expression::StaticMemberExpression(member) = &call.callee else {
        return None;
    };

    let Expression::Identifier(obj) = &member.object else {
        return None;
    };

    if obj.name != "Promise" {
        return None;
    }

    let name = member.property.name.as_str();
    (name == "resolve" || name == "reject").then_some(name)
}

/// Find the semantic [`NodeId`] for a node with the given span.
fn find_node_id_by_span(
    semantic: &oxc_semantic::Semantic<'_>,
    span: oxc_span::Span,
) -> Option<oxc_semantic::NodeId> {
    for node in semantic.nodes() {
        if node.kind().span() == span {
            return Some(node.id());
        }
    }
    None
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUselessPromiseResolveReject)];
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
    fn test_flags_resolve_in_async() {
        let diags = lint("async function foo() { return Promise.resolve(1); }");
        assert_eq!(diags.len(), 1, "Promise.resolve in async should be flagged");
    }

    #[test]
    fn test_flags_reject_in_async() {
        let diags = lint("async function foo() { return Promise.reject(new Error('x')); }");
        assert_eq!(diags.len(), 1, "Promise.reject in async should be flagged");
    }

    #[test]
    fn test_allows_resolve_in_non_async() {
        let diags = lint("function foo() { return Promise.resolve(1); }");
        assert!(
            diags.is_empty(),
            "Promise.resolve in non-async should not be flagged"
        );
    }

    #[test]
    fn test_allows_normal_return_in_async() {
        let diags = lint("async function foo() { return 1; }");
        assert!(
            diags.is_empty(),
            "normal return in async should not be flagged"
        );
    }

    #[test]
    fn test_allows_promise_all() {
        let diags = lint("async function foo() { return Promise.all([a, b]); }");
        assert!(
            diags.is_empty(),
            "Promise.all in async should not be flagged"
        );
    }
}
