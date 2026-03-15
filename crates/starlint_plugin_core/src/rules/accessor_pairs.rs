//! Rule: `accessor-pairs`
//!
//! Require matching getter and setter pairs in object literals and class
//! bodies. A setter without a corresponding getter is usually a mistake,
//! because the value can be set but never retrieved.
//!
//! By default (matching `ESLint`), only setters without getters are flagged.
//! Getters without setters are allowed.

use std::collections::HashMap;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::{AstNode, ClassNode, ObjectExpressionNode};
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::{MethodDefinitionKind, PropertyKind};
use starlint_ast::types::NodeId;
use starlint_rule_framework::ast_utils::extract_static_key_name;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags setters without a corresponding getter.
#[derive(Debug)]
pub struct AccessorPairs;

impl LintRule for AccessorPairs {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "accessor-pairs".to_owned(),
            description: "Require matching getter/setter pairs".to_owned(),
            category: Category::Suggestion,
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

/// Tracks whether a getter and/or setter exists for a property name.
struct PairInfo {
    /// Whether a getter exists for this property.
    has_getter: bool,
    /// Whether a setter exists for this property.
    has_setter: bool,
    /// Span of the setter (used for the diagnostic location when getter is missing).
    setter_span: Option<Span>,
}

/// Check accessor pairs in an object expression.
fn check_object_expression(obj: &ObjectExpressionNode, ctx: &mut LintContext<'_>) {
    let mut pairs: HashMap<String, PairInfo> = HashMap::new();

    for &prop_id in &*obj.properties {
        let Some(AstNode::ObjectProperty(prop)) = ctx.node(prop_id) else {
            continue;
        };

        if prop.kind != PropertyKind::Get && prop.kind != PropertyKind::Set {
            continue;
        }

        // Skip computed properties
        if prop.computed {
            continue;
        }

        let Some(name) = extract_static_key_name(prop.key, ctx) else {
            continue;
        };

        let prop_kind = prop.kind;
        let prop_span = prop.span;

        let entry = pairs.entry(name).or_insert_with(|| PairInfo {
            has_getter: false,
            has_setter: false,
            setter_span: None,
        });

        if prop_kind == PropertyKind::Get {
            entry.has_getter = true;
        } else {
            entry.has_setter = true;
            entry.setter_span = Some(Span::new(prop_span.start, prop_span.end));
        }
    }

    report_missing_getters(&pairs, ctx);
}

/// Check accessor pairs in a class body.
fn check_class_body(class: &ClassNode, ctx: &mut LintContext<'_>) {
    // Key: (is_static, name)
    let mut pairs: HashMap<(bool, String), PairInfo> = HashMap::new();

    for &element_id in &*class.body {
        let Some(AstNode::MethodDefinition(method)) = ctx.node(element_id) else {
            continue;
        };

        if method.kind != MethodDefinitionKind::Get && method.kind != MethodDefinitionKind::Set {
            continue;
        }

        // Skip computed properties
        if method.computed {
            continue;
        }

        let Some(name) = extract_static_key_name(method.key, ctx) else {
            continue;
        };

        let method_kind = method.kind;
        let method_span = method.span;
        let is_static = method.is_static;

        let key = (is_static, name);
        let entry = pairs.entry(key).or_insert_with(|| PairInfo {
            has_getter: false,
            has_setter: false,
            setter_span: None,
        });

        if method_kind == MethodDefinitionKind::Get {
            entry.has_getter = true;
        } else {
            entry.has_setter = true;
            entry.setter_span = Some(Span::new(method_span.start, method_span.end));
        }
    }

    report_missing_getters_keyed(&pairs, ctx);
}

/// Report setters that have no matching getter (string keys).
fn report_missing_getters(pairs: &HashMap<String, PairInfo>, ctx: &mut LintContext<'_>) {
    for (name, info) in pairs {
        if info.has_setter && !info.has_getter {
            if let Some(span) = info.setter_span {
                ctx.report(Diagnostic {
                    rule_name: "accessor-pairs".to_owned(),
                    message: format!("Setter for `{name}` has no corresponding getter"),
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

/// Report setters that have no matching getter (static+name keys).
fn report_missing_getters_keyed(
    pairs: &HashMap<(bool, String), PairInfo>,
    ctx: &mut LintContext<'_>,
) {
    for ((_, name), info) in pairs {
        if info.has_setter && !info.has_getter {
            if let Some(span) = info.setter_span {
                ctx.report(Diagnostic {
                    rule_name: "accessor-pairs".to_owned(),
                    message: format!("Setter for `{name}` has no corresponding getter"),
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

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(AccessorPairs);

    #[test]
    fn test_allows_complete_pair() {
        let diags = lint("const obj = { get foo() {}, set foo(v) {} }");
        assert!(
            diags.is_empty(),
            "complete getter/setter pair should not be flagged"
        );
    }

    #[test]
    fn test_flags_setter_without_getter() {
        let diags = lint("const obj = { set foo(v) {} }");
        assert_eq!(diags.len(), 1, "setter without getter should be flagged");
    }

    #[test]
    fn test_allows_getter_without_setter() {
        let diags = lint("const obj = { get foo() {} }");
        assert!(
            diags.is_empty(),
            "getter without setter should not be flagged (ESLint default)"
        );
    }

    #[test]
    fn test_allows_class_complete_pair() {
        let diags = lint("class C { get x() {} set x(v) {} }");
        assert!(
            diags.is_empty(),
            "class with complete getter/setter pair should not be flagged"
        );
    }

    #[test]
    fn test_flags_class_setter_without_getter() {
        let diags = lint("class C { set x(v) {} }");
        assert_eq!(
            diags.len(),
            1,
            "class setter without getter should be flagged"
        );
    }

    #[test]
    fn test_allows_class_getter_without_setter() {
        let diags = lint("class C { get x() {} }");
        assert!(
            diags.is_empty(),
            "class getter without setter should not be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_incomplete_pairs() {
        let diags = lint("const obj = { set foo(v) {}, set bar(v) {} }");
        assert_eq!(
            diags.len(),
            2,
            "two setters without getters should produce two diagnostics"
        );
    }

    #[test]
    fn test_allows_normal_properties() {
        let diags = lint("const obj = { foo: 1, bar: 2 }");
        assert!(diags.is_empty(), "normal properties should not be flagged");
    }

    #[test]
    fn test_flags_static_setter_without_getter() {
        let diags = lint("class C { static set x(v) {} }");
        assert_eq!(
            diags.len(),
            1,
            "static setter without getter should be flagged"
        );
    }
}
