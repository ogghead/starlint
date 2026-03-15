//! Rule: `no-thenable` (unicorn)
//!
//! Disallow the use of `then` as a property name on objects/classes.
//! Objects with a `then` method are treated as "thenables" by the
//! Promise system, which can cause unexpected behavior.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags objects and classes that define a `then` property or method.
#[derive(Debug)]
pub struct NoThenable;

impl LintRule for NoThenable {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-thenable".to_owned(),
            description: "Disallow then property".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::MethodDefinition,
            AstNodeType::ObjectExpression,
            AstNodeType::PropertyDefinition,
        ])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            // Check object properties: { then: ... } or { then() {} }
            AstNode::ObjectExpression(obj) => {
                for prop_id in &obj.properties {
                    let Some(AstNode::ObjectProperty(p)) = ctx.node(*prop_id) else {
                        continue;
                    };
                    if property_key_is_then(p.key, ctx) {
                        ctx.report(Diagnostic {
                            rule_name: "no-thenable".to_owned(),
                            message: "Do not add `then` to an object".to_owned(),
                            span: Span::new(p.span.start, p.span.end),
                            severity: Severity::Warning,
                            help: None,
                            fix: None,
                            labels: vec![],
                        });
                    }
                }
            }
            // Check class methods/properties named `then`
            AstNode::MethodDefinition(method) => {
                if property_key_is_then(method.key, ctx) {
                    ctx.report(Diagnostic {
                        rule_name: "no-thenable".to_owned(),
                        message: "Do not add `then` to a class".to_owned(),
                        span: Span::new(method.span.start, method.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            AstNode::PropertyDefinition(prop) => {
                if property_key_is_then(prop.key, ctx) {
                    ctx.report(Diagnostic {
                        rule_name: "no-thenable".to_owned(),
                        message: "Do not add `then` to a class".to_owned(),
                        span: Span::new(prop.span.start, prop.span.end),
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
}

/// Check if a property key node is the identifier or string `"then"`.
fn property_key_is_then(key_id: NodeId, ctx: &LintContext<'_>) -> bool {
    match ctx.node(key_id) {
        Some(AstNode::IdentifierReference(id)) => id.name == "then",
        Some(AstNode::BindingIdentifier(id)) => id.name == "then",
        Some(AstNode::StringLiteral(s)) => s.value == "then",
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoThenable);

    #[test]
    fn test_flags_object_then_property() {
        let diags = lint("var obj = { then: function() {} };");
        assert_eq!(
            diags.len(),
            1,
            "object with then property should be flagged"
        );
    }

    #[test]
    fn test_flags_object_then_method() {
        let diags = lint("var obj = { then() {} };");
        assert_eq!(diags.len(), 1, "object with then method should be flagged");
    }

    #[test]
    fn test_flags_class_then_method() {
        let diags = lint("class Foo { then() {} }");
        assert_eq!(diags.len(), 1, "class with then method should be flagged");
    }

    #[test]
    fn test_allows_other_property() {
        let diags = lint("var obj = { foo: 1 };");
        assert!(diags.is_empty(), "other properties should not be flagged");
    }

    #[test]
    fn test_allows_other_method() {
        let diags = lint("class Foo { bar() {} }");
        assert!(diags.is_empty(), "other methods should not be flagged");
    }
}
