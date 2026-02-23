//! Rule: `jest/prefer-to-be`
//!
//! Suggest `expect(x).toBe(y)` over `expect(x).toEqual(y)` for primitive
//! literal values. `toBe` uses `Object.is` which is more appropriate and
//! faster for primitives than the deep-equality check of `toEqual`.

#![allow(clippy::shadow_unrelated)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `expect(x).toEqual(primitive)` patterns that should use `toBe`.
#[derive(Debug)]
pub struct PreferToBe;

impl LintRule for PreferToBe {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-to-be".to_owned(),
            description: "Suggest using `toBe()` for primitive literals instead of `toEqual()`"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        source_text.contains("toEqual(") && crate::is_test_file(file_path)
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(
        clippy::arithmetic_side_effects,
        clippy::cast_possible_truncation,
        clippy::shadow_unrelated
    )]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Check that callee is a member expression with `.toEqual`
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };
        if member.property.as_str() != "toEqual" {
            return;
        }

        // The object should be an `expect(...)` call (or chained `.not.toEqual`)
        if !is_expect_chain(ctx, member.object) {
            return;
        }

        // Check that the first argument to `toEqual` is a primitive literal
        let Some(first_arg_id) = call.arguments.first() else {
            return;
        };
        let Some(arg_node) = ctx.node(*first_arg_id) else {
            return;
        };
        if is_primitive_literal(arg_node) {
            // We need to build the fix span for the property name.
            // Since member.property is a String (no span), we use the source text
            // to find "toEqual" within the call span and replace the whole call's
            // method name. A simpler approach: replace the entire call expression
            // by substituting "toEqual" with "toBe" in the source text.
            let source = ctx.source_text();
            #[allow(clippy::as_conversions)]
            let call_text = source
                .get(call.span.start as usize..call.span.end as usize)
                .unwrap_or("");
            // Find "toEqual" in the call text and replace with "toBe"
            if let Some(idx) = call_text.find("toEqual") {
                #[allow(clippy::as_conversions)]
                let prop_start = call.span.start + idx as u32;
                #[allow(clippy::as_conversions)]
                let prop_end = prop_start + "toEqual".len() as u32;
                let prop_span = Span::new(prop_start, prop_end);

                ctx.report(Diagnostic {
                    rule_name: "jest/prefer-to-be".to_owned(),
                    message: "Use `toBe` instead of `toEqual` when comparing primitive values"
                        .to_owned(),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Warning,
                    help: Some("Replace `toEqual` with `toBe`".to_owned()),
                    fix: Some(Fix {
                        kind: FixKind::SafeFix,
                        message: "Replace with `toBe`".to_owned(),
                        edits: vec![Edit {
                            span: prop_span,
                            replacement: "toBe".to_owned(),
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
            }
        }
    }
}

/// Check whether an `AstNode` is a primitive literal (string, number, boolean,
/// null, undefined).
fn is_primitive_literal(node: &AstNode) -> bool {
    matches!(
        node,
        AstNode::StringLiteral(_)
            | AstNode::NumericLiteral(_)
            | AstNode::BooleanLiteral(_)
            | AstNode::NullLiteral(_)
    ) || is_undefined(node)
}

/// Check if the node is the identifier `undefined`.
fn is_undefined(node: &AstNode) -> bool {
    matches!(node, AstNode::IdentifierReference(id) if id.name.as_str() == "undefined")
}

/// Check if a node (by ID) is an `expect(...)` call or a chain like
/// `expect(...).not`.
fn is_expect_chain(ctx: &LintContext<'_>, id: NodeId) -> bool {
    let Some(node) = ctx.node(id) else {
        return false;
    };
    match node {
        AstNode::CallExpression(call) => ctx.node(call.callee).is_some_and(
            |n| matches!(n, AstNode::IdentifierReference(id) if id.name.as_str() == "expect"),
        ),
        AstNode::StaticMemberExpression(member) => is_expect_chain(ctx, member.object),
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferToBe)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_to_equal_with_number() {
        let diags = lint("expect(x).toEqual(1);");
        assert_eq!(
            diags.len(),
            1,
            "`toEqual(1)` should be flagged as prefer `toBe`"
        );
    }

    #[test]
    fn test_flags_to_equal_with_string() {
        let diags = lint(r#"expect(x).toEqual("hello");"#);
        assert_eq!(
            diags.len(),
            1,
            "`toEqual(\"hello\")` should be flagged as prefer `toBe`"
        );
    }

    #[test]
    fn test_allows_to_equal_with_object() {
        let diags = lint("expect(x).toEqual({ a: 1 });");
        assert!(
            diags.is_empty(),
            "`toEqual` with an object literal should not be flagged"
        );
    }
}
