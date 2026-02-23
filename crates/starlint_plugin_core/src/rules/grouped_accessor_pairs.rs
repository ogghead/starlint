//! Rule: `grouped-accessor-pairs`
//!
//! Require getter and setter pairs for the same property to be adjacent in
//! object literals and class bodies. Separating a getter and setter with
//! unrelated properties makes the code harder to read and maintain.

use std::collections::HashMap;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::{AstNode, ClassNode, ObjectExpressionNode};
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::{MethodDefinitionKind, PropertyKind};
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags getter/setter pairs that are not adjacent.
#[derive(Debug)]
pub struct GroupedAccessorPairs;

impl LintRule for GroupedAccessorPairs {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "grouped-accessor-pairs".to_owned(),
            description: "Require grouped getter/setter pairs in objects and classes".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::Class, AstNodeType::ObjectExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::ObjectExpression(obj) => {
                check_object_expression(obj, ctx);
            }
            AstNode::Class(class) => {
                check_class_body(class, ctx);
            }
            _ => {}
        }
    }
}

/// Tracks the position and span of an accessor (getter or setter).
struct AccessorInfo {
    /// Position of the getter in the property list.
    getter_pos: Option<usize>,
    /// Position of the setter in the property list.
    setter_pos: Option<usize>,
    /// Span of the second accessor found (used for the diagnostic location).
    second_span: Option<Span>,
}

/// Check getter/setter adjacency in an object expression.
fn check_object_expression(obj: &ObjectExpressionNode, ctx: &mut LintContext<'_>) {
    let mut accessors: HashMap<String, AccessorInfo> = HashMap::new();

    for (i, &prop_id) in obj.properties.iter().enumerate() {
        let Some(AstNode::ObjectProperty(prop)) = ctx.node(prop_id) else {
            continue;
        };

        // Only interested in getters and setters
        if prop.kind != PropertyKind::Get && prop.kind != PropertyKind::Set {
            continue;
        }

        // Skip computed properties — can't determine name statically
        if prop.computed {
            continue;
        }

        let Some(name) = static_property_key_name(prop.key, ctx) else {
            continue;
        };

        let prop_kind = prop.kind;
        let prop_span = prop.span;

        let entry = accessors.entry(name).or_insert_with(|| AccessorInfo {
            getter_pos: None,
            setter_pos: None,
            second_span: None,
        });

        if prop_kind == PropertyKind::Get {
            entry.getter_pos = Some(i);
        } else {
            entry.setter_pos = Some(i);
        }

        // If both are now set, record the span of this (second) accessor
        if entry.getter_pos.is_some() && entry.setter_pos.is_some() && entry.second_span.is_none() {
            entry.second_span = Some(Span::new(prop_span.start, prop_span.end));
        }
    }

    report_non_adjacent(&accessors, ctx);
}

/// Check getter/setter adjacency in a class body.
fn check_class_body(class: &ClassNode, ctx: &mut LintContext<'_>) {
    let mut accessors: HashMap<(bool, String), AccessorInfo> = HashMap::new();

    for (i, &element_id) in class.body.iter().enumerate() {
        let Some(AstNode::MethodDefinition(method)) = ctx.node(element_id) else {
            continue;
        };

        // Only interested in getters and setters
        if method.kind != MethodDefinitionKind::Get && method.kind != MethodDefinitionKind::Set {
            continue;
        }

        // Skip computed properties
        if method.computed {
            continue;
        }

        let Some(name) = static_property_key_name(method.key, ctx) else {
            continue;
        };

        let method_kind = method.kind;
        let method_span = method.span;
        let is_static = method.is_static;

        let key = (is_static, name);
        let entry = accessors.entry(key).or_insert_with(|| AccessorInfo {
            getter_pos: None,
            setter_pos: None,
            second_span: None,
        });

        if method_kind == MethodDefinitionKind::Get {
            entry.getter_pos = Some(i);
        } else {
            entry.setter_pos = Some(i);
        }

        // If both are now set, record the span of this (second) accessor
        if entry.getter_pos.is_some() && entry.setter_pos.is_some() && entry.second_span.is_none() {
            entry.second_span = Some(Span::new(method_span.start, method_span.end));
        }
    }

    report_non_adjacent_keyed(&accessors, ctx);
}

/// Report diagnostics for non-adjacent getter/setter pairs (string keys).
fn report_non_adjacent(accessors: &HashMap<String, AccessorInfo>, ctx: &mut LintContext<'_>) {
    for (name, info) in accessors {
        let (Some(getter_pos), Some(setter_pos)) = (info.getter_pos, info.setter_pos) else {
            continue;
        };

        let diff = getter_pos.abs_diff(setter_pos);
        if diff > 1 {
            if let Some(span) = info.second_span {
                ctx.report(Diagnostic {
                    rule_name: "grouped-accessor-pairs".to_owned(),
                    message: format!("Getter and setter for `{name}` should be grouped together"),
                    span,
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
        }
    }
}

/// Report diagnostics for non-adjacent getter/setter pairs (static+name keys).
fn report_non_adjacent_keyed(
    accessors: &HashMap<(bool, String), AccessorInfo>,
    ctx: &mut LintContext<'_>,
) {
    for ((_, name), info) in accessors {
        let (Some(getter_pos), Some(setter_pos)) = (info.getter_pos, info.setter_pos) else {
            continue;
        };

        let diff = getter_pos.abs_diff(setter_pos);
        if diff > 1 {
            if let Some(span) = info.second_span {
                ctx.report(Diagnostic {
                    rule_name: "grouped-accessor-pairs".to_owned(),
                    message: format!("Getter and setter for `{name}` should be grouped together"),
                    span,
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
        }
    }
}

/// Extract a static key name from a property key `NodeId`.
fn static_property_key_name(key: NodeId, ctx: &LintContext<'_>) -> Option<String> {
    match ctx.node(key)? {
        AstNode::IdentifierReference(ident) => Some(ident.name.clone()),
        AstNode::BindingIdentifier(ident) => Some(ident.name.clone()),
        AstNode::StringLiteral(lit) => Some(lit.value.clone()),
        AstNode::NumericLiteral(lit) => Some(lit.raw.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(GroupedAccessorPairs)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_allows_adjacent_object_accessors() {
        let diags = lint("const obj = { get foo() {}, set foo(v) {} }");
        assert!(
            diags.is_empty(),
            "adjacent getter/setter pair should not be flagged"
        );
    }

    #[test]
    fn test_flags_non_adjacent_object_accessors() {
        let diags = lint("const obj = { get foo() {}, bar: 1, set foo(v) {} }");
        assert_eq!(
            diags.len(),
            1,
            "non-adjacent getter/setter pair should be flagged"
        );
    }

    #[test]
    fn test_allows_adjacent_class_accessors() {
        let diags = lint("class C { get foo() {} set foo(v) {} }");
        assert!(
            diags.is_empty(),
            "adjacent class getter/setter should not be flagged"
        );
    }

    #[test]
    fn test_flags_non_adjacent_class_accessors() {
        let diags = lint("class C { get foo() {} bar() {} set foo(v) {} }");
        assert_eq!(
            diags.len(),
            1,
            "non-adjacent class getter/setter should be flagged"
        );
    }

    #[test]
    fn test_allows_only_getter() {
        let diags = lint("const obj = { get foo() {} }");
        assert!(diags.is_empty(), "lone getter should not be flagged");
    }

    #[test]
    fn test_allows_only_setter() {
        let diags = lint("const obj = { set foo(v) {} }");
        assert!(diags.is_empty(), "lone setter should not be flagged");
    }

    #[test]
    fn test_allows_reversed_adjacent() {
        let diags = lint("const obj = { set foo(v) {}, get foo() {} }");
        assert!(
            diags.is_empty(),
            "reversed but adjacent getter/setter should not be flagged"
        );
    }

    #[test]
    fn test_flags_static_non_adjacent() {
        let diags = lint("class C { static get x() {} bar() {} static set x(v) {} }");
        assert_eq!(
            diags.len(),
            1,
            "non-adjacent static getter/setter should be flagged"
        );
    }
}
