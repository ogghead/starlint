//! Rule: `typescript/prefer-optional-chain`
//!
//! Prefer optional chaining (`foo?.bar`) over short-circuit evaluation
//! (`foo && foo.bar`). The `&&` pattern is verbose and error-prone compared
//! to the optional chaining operator introduced in ES2020.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::LogicalOperator;
use starlint_ast::types::NodeId;

/// Flags `foo && foo.bar` patterns that can use optional chaining.
#[derive(Debug)]
pub struct PreferOptionalChain;

impl LintRule for PreferOptionalChain {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/prefer-optional-chain".to_owned(),
            description: "Prefer `?.` optional chaining over `&&` short-circuit guards".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::LogicalExpression])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::LogicalExpression(logical) = node else {
            return;
        };

        if logical.operator != LogicalOperator::And {
            return;
        }

        // Left side must be a plain identifier
        let Some(AstNode::IdentifierReference(left_id)) = ctx.node(logical.left) else {
            return;
        };
        let guard_name = left_id.name.clone();

        // Right side must be a member expression or call on that same identifier
        if !is_member_or_call_on(&guard_name, logical.right, ctx) {
            return;
        }

        // Build fix: replace `foo && foo.bar` with `foo?.bar`
        // by inserting `?` after the first `.` in the right side
        let right_span = ctx.node(logical.right).map(starlint_ast::AstNode::span);
        let fix = right_span.and_then(|rs| {
            let source = ctx.source_text();
            let right_text = source.get(rs.start as usize..rs.end as usize)?;
            // Replace "foo." with "foo?." at the start of the right expression
            let prefix = format!("{guard_name}.");
            right_text.starts_with(prefix.as_str()).then(|| {
                let replacement = format!("{guard_name}?.{}", &right_text[prefix.len()..]);
                Fix {
                    kind: FixKind::SuggestionFix,
                    message: format!("Replace with `{replacement}`"),
                    edits: vec![Edit {
                        span: Span::new(logical.span.start, logical.span.end),
                        replacement,
                    }],
                    is_snippet: false,
                }
            })
        });

        ctx.report(Diagnostic {
            rule_name: "typescript/prefer-optional-chain".to_owned(),
            message: format!(
                "Prefer `{guard_name}?.` optional chaining over `{guard_name} && {guard_name}.\u{2026}`"
            ),
            span: Span::new(logical.span.start, logical.span.end),
            severity: Severity::Warning,
            help: Some("Use optional chaining operator `?.`".to_owned()),
            fix,
            labels: vec![],
        });
    }
}

/// Check if the expression is a member access or call on the given identifier.
///
/// Matches patterns like `foo.bar`, `foo.bar()`, `foo["bar"]`, or `foo.bar.baz`.
fn is_member_or_call_on(name: &str, expr_id: NodeId, ctx: &LintContext<'_>) -> bool {
    match ctx.node(expr_id) {
        Some(AstNode::StaticMemberExpression(member)) => {
            object_matches_name(name, member.object, ctx)
        }
        Some(AstNode::ComputedMemberExpression(member)) => {
            object_matches_name(name, member.object, ctx)
        }
        Some(AstNode::CallExpression(call)) => {
            // Check if callee is a member expression on the same identifier
            // e.g. `foo.bar()` where callee is `foo.bar`
            match ctx.node(call.callee) {
                Some(AstNode::StaticMemberExpression(member)) => {
                    object_matches_name(name, member.object, ctx)
                }
                Some(AstNode::ComputedMemberExpression(member)) => {
                    object_matches_name(name, member.object, ctx)
                }
                _ => false,
            }
        }
        _ => false,
    }
}

/// Check if the object of a member expression is the given identifier name.
///
/// Handles both direct identifier (`foo.bar`) and chained member expressions
/// (`foo.bar.baz` by checking the root object).
#[allow(clippy::as_conversions)]
fn object_matches_name(name: &str, object_id: NodeId, ctx: &LintContext<'_>) -> bool {
    match ctx.node(object_id) {
        Some(AstNode::IdentifierReference(id)) => id.name.as_str() == name,
        // For chained access like `foo.bar.baz`, check the root object recursively
        Some(AstNode::StaticMemberExpression(member)) => {
            object_matches_name(name, member.object, ctx)
        }
        Some(AstNode::ComputedMemberExpression(member)) => {
            let source = ctx.source_text();
            let obj_span = ctx.node(member.object).map(starlint_ast::AstNode::span);
            obj_span
                .and_then(|s| source.get(s.start as usize..s.end as usize))
                .is_some_and(|s| s == name)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferOptionalChain)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_and_member_access() {
        let diags = lint("declare const foo: any; foo && foo.bar;");
        assert_eq!(diags.len(), 1, "`foo && foo.bar` should be flagged");
    }

    #[test]
    fn test_flags_and_method_call() {
        let diags = lint("declare const foo: any; foo && foo.baz();");
        assert_eq!(diags.len(), 1, "`foo && foo.baz()` should be flagged");
    }

    #[test]
    fn test_allows_optional_chaining() {
        let diags = lint("declare const foo: any; foo?.bar;");
        assert!(diags.is_empty(), "`foo?.bar` should not be flagged");
    }

    #[test]
    fn test_allows_different_identifiers() {
        let diags = lint("declare const foo: any; declare const bar: any; foo && bar.baz;");
        assert!(
            diags.is_empty(),
            "`foo && bar.baz` should not be flagged (different identifiers)"
        );
    }

    #[test]
    fn test_allows_or_operator() {
        let diags = lint("declare const foo: any; foo || foo.bar;");
        assert!(diags.is_empty(), "`||` operator should not be flagged");
    }

    #[test]
    fn test_allows_non_member_right() {
        let diags = lint("declare const foo: any; foo && true;");
        assert!(diags.is_empty(), "`foo && true` should not be flagged");
    }
}
