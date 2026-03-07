//! Rule: `typescript/no-unsafe-argument`
//!
//! Disallow calling a function with an `any`-typed argument. Passing `as any`
//! to a function defeats type checking for that parameter position and can
//! hide type errors.
//!
//! Simplified syntax-only version — full checking requires type information.
//! This AST-based rule only detects `as any` expressions passed directly as
//! function call arguments (e.g. `foo(x as any)`).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "typescript/no-unsafe-argument";

/// Flags function call arguments that use `as any` type assertions.
#[derive(Debug)]
pub struct NoUnsafeArgument;

impl LintRule for NoUnsafeArgument {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow calling a function with an `as any` argument".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        for arg_id in &call.arguments {
            if is_as_any_argument(*arg_id, ctx) {
                let arg_span = ctx.node(*arg_id).map_or(
                    starlint_ast::types::Span::EMPTY,
                    starlint_ast::AstNode::span,
                );
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: "Unsafe `as any` argument — this bypasses type checking for \
                     the corresponding parameter"
                        .to_owned(),
                    span: Span::new(arg_span.start, arg_span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
        }
    }
}

/// Check if a function argument is a `TSAsExpression` casting to `any`.
/// `TSAsExpressionNode` has no `type_annotation` — use source text heuristic.
fn is_as_any_argument(node_id: NodeId, ctx: &LintContext<'_>) -> bool {
    let Some(AstNode::TSAsExpression(ts_as)) = ctx.node(node_id) else {
        return false;
    };
    // Check if the source text of the as-expression ends with "as any"
    let source = ctx.source_text();
    let start = usize::try_from(ts_as.span.start).unwrap_or(0);
    let end = usize::try_from(ts_as.span.end).unwrap_or(0);
    source
        .get(start..end)
        .is_some_and(|text| text.trim_end().ends_with("as any"))
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUnsafeArgument)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_as_any_argument() {
        let diags = lint("declare function foo(x: number): void;\nfoo(value as any);");
        assert_eq!(diags.len(), 1, "`as any` argument should be flagged");
    }

    #[test]
    fn test_flags_multiple_as_any_arguments() {
        let diags =
            lint("declare function bar(a: string, b: number): void;\nbar(x as any, y as any);");
        assert_eq!(diags.len(), 2, "both `as any` arguments should be flagged");
    }

    #[test]
    fn test_allows_as_string_argument() {
        let diags = lint("declare function foo(x: string): void;\nfoo(value as string);");
        assert!(
            diags.is_empty(),
            "`as string` argument should not be flagged"
        );
    }

    #[test]
    fn test_allows_normal_argument() {
        let diags = lint("declare function foo(x: number): void;\nfoo(42);");
        assert!(diags.is_empty(), "normal argument should not be flagged");
    }

    #[test]
    fn test_allows_as_unknown_argument() {
        let diags = lint("declare function foo(x: unknown): void;\nfoo(value as unknown);");
        assert!(
            diags.is_empty(),
            "`as unknown` argument should not be flagged by this rule"
        );
    }
}
