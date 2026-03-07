//! Rule: `react/jsx-key`
//!
//! Warn when JSX elements in array `.map()` calls are missing a `key` prop.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-key";

/// Warns when JSX elements returned from `.map()` callbacks lack a `key` prop.
#[derive(Debug)]
pub struct JsxKey;

/// Check whether a JSX opening element has a `key` attribute.
fn has_key_prop(ctx: &LintContext<'_>, opening_id: NodeId) -> bool {
    let Some(AstNode::JSXOpeningElement(opening)) = ctx.node(opening_id) else {
        return false;
    };
    opening.attributes.iter().any(|attr_id| {
        let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) else {
            return false;
        };
        attr.name.as_str() == "key"
    })
}

/// Check whether a node (by ID) is a JSX element or fragment without a key.
fn is_jsx_without_key(ctx: &LintContext<'_>, id: NodeId) -> bool {
    let Some(node) = ctx.node(id) else {
        return false;
    };
    match node {
        AstNode::JSXElement(el) => !has_key_prop(ctx, el.opening_element),
        AstNode::JSXFragment(_) => true,
        _ => false,
    }
}

/// Check if a callback argument (by ID) returns JSX without a `key` prop.
fn callback_returns_jsx_without_key(ctx: &LintContext<'_>, callback_id: NodeId) -> bool {
    let Some(callback) = ctx.node(callback_id) else {
        return false;
    };
    match callback {
        AstNode::ArrowFunctionExpression(arrow) => {
            let Some(AstNode::FunctionBody(body)) = ctx.node(arrow.body) else {
                return false;
            };
            if arrow.expression {
                // Arrow with expression body: `items.map(x => <div />)`
                if let Some(first_stmt_id) = body.statements.first() {
                    if let Some(AstNode::ExpressionStatement(expr_stmt)) = ctx.node(*first_stmt_id)
                    {
                        return is_jsx_without_key(ctx, expr_stmt.expression);
                    }
                }
            }
            // Arrow with block body: check return statements
            for stmt_id in &*body.statements {
                if let Some(AstNode::ReturnStatement(ret)) = ctx.node(*stmt_id) {
                    if let Some(ret_val) = ret.argument {
                        if is_jsx_without_key(ctx, ret_val) {
                            return true;
                        }
                    }
                }
            }
            false
        }
        AstNode::Function(func) => {
            let Some(body_id) = func.body else {
                return false;
            };
            let Some(AstNode::FunctionBody(body)) = ctx.node(body_id) else {
                return false;
            };
            for stmt_id in &*body.statements {
                if let Some(AstNode::ReturnStatement(ret)) = ctx.node(*stmt_id) {
                    if let Some(ret_val) = ret.argument {
                        if is_jsx_without_key(ctx, ret_val) {
                            return true;
                        }
                    }
                }
            }
            false
        }
        _ => false,
    }
}

impl LintRule for JsxKey {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Warn when JSX elements in `.map()` calls are missing a `key` prop"
                .to_owned(),
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

        // Check if callee is `<expr>.map`
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property.as_str() != "map" {
            return;
        }

        // Check the first argument (the callback)
        let Some(first_arg_id) = call.arguments.first() else {
            return;
        };

        if callback_returns_jsx_without_key(ctx, *first_arg_id) {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Missing `key` prop for JSX element in `.map()` iterator".to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(JsxKey)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_missing_key_in_map() {
        let diags = lint("const items = [1,2].map(x => <div>{x}</div>);");
        assert_eq!(diags.len(), 1, "should flag JSX without key in .map()");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_key_present() {
        let diags = lint("const items = [1,2].map(x => <div key={x}>{x}</div>);");
        assert!(diags.is_empty(), "should not flag when key prop is present");
    }

    #[test]
    fn test_flags_block_body_missing_key() {
        let diags = lint("const items = [1,2].map(x => { return <li>{x}</li>; });");
        assert_eq!(
            diags.len(),
            1,
            "should flag JSX without key in block-body .map()"
        );
    }
}
