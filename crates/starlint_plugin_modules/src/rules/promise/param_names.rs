//! Rule: `promise/param-names`
//!
//! Enforce standard `resolve`/`reject` parameter names in `new Promise()`
//! executors. Consistent naming improves readability.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `new Promise()` executors whose parameters are not named
/// `resolve` and `reject`.
#[derive(Debug)]
pub struct ParamNames;

impl LintRule for ParamNames {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/param-names".to_owned(),
            description: "Enforce standard `resolve`/`reject` parameter names".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::NewExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::NewExpression(new_expr) = node else {
            return;
        };

        let Some(AstNode::IdentifierReference(ident)) = ctx.node(new_expr.callee) else {
            return;
        };

        if ident.name.as_str() != "Promise" {
            return;
        }

        let Some(first_arg_id) = new_expr.arguments.first() else {
            return;
        };

        // Skip spread elements
        if matches!(ctx.node(*first_arg_id), Some(AstNode::SpreadElement(_))) {
            return;
        }

        // Extract parameter NodeIds from the executor function
        let params: &[NodeId] = match ctx.node(*first_arg_id) {
            Some(AstNode::ArrowFunctionExpression(arrow)) => &arrow.params,
            Some(AstNode::Function(func)) => &func.params,
            _ => return,
        };

        // Collect param info to avoid borrow conflicts with ctx.report()
        let mut violations: Vec<(String, starlint_ast::types::Span, &str)> = Vec::new();

        // Check first parameter (resolve)
        if let Some(first_id) = params.first() {
            if let Some(AstNode::BindingIdentifier(id)) = ctx.node(*first_id) {
                let name = id.name.as_str();
                if name != "resolve" && name != "_resolve" && name != "_" {
                    violations.push((name.to_owned(), id.span, "resolve"));
                }
            }
        }

        // Check second parameter (reject)
        if let Some(second_id) = params.get(1) {
            if let Some(AstNode::BindingIdentifier(id)) = ctx.node(*second_id) {
                let name = id.name.as_str();
                if name != "reject" && name != "_reject" && name != "_" {
                    violations.push((name.to_owned(), id.span, "reject"));
                }
            }
        }

        for (name, span, expected) in violations {
            ctx.report(Diagnostic {
                rule_name: "promise/param-names".to_owned(),
                message: format!(
                    "Promise executor parameter should be named `{expected}`, found `{name}`"
                ),
                span: Span::new(span.start, span.end),
                severity: Severity::Warning,
                help: None,
                fix: Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: format!("Rename to `{expected}`"),
                    edits: vec![Edit {
                        span: Span::new(span.start, span.end),
                        replacement: expected.to_owned(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(ParamNames);

    #[test]
    fn test_flags_non_standard_names() {
        let diags = lint("new Promise((yes, no) => { yes(1); });");
        assert_eq!(diags.len(), 2, "should flag both non-standard param names");
    }

    #[test]
    fn test_allows_standard_names() {
        let diags = lint("new Promise((resolve, reject) => { resolve(1); });");
        assert!(diags.is_empty(), "standard names should be allowed");
    }

    #[test]
    fn test_allows_underscore_prefix() {
        let diags = lint("new Promise((_resolve, _reject) => { });");
        assert!(
            diags.is_empty(),
            "underscore-prefixed names should be allowed"
        );
    }
}
