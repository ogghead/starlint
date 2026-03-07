//! Rule: `typescript/no-extraneous-class`
//!
//! Disallow classes that contain only static members or are empty. Such classes
//! add no value over plain objects or module-level functions and exports. If a
//! class has a constructor, extends another class, or contains any instance
//! members, it is considered valid.

#![allow(clippy::match_same_arms)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::MethodDefinitionKind;
use starlint_ast::types::NodeId;

/// Flags classes that are empty or contain only static members.
#[derive(Debug)]
pub struct NoExtraneousClass;

impl LintRule for NoExtraneousClass {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-extraneous-class".to_owned(),
            description: "Disallow classes with only static members or empty bodies".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::Class])
    }

    #[allow(clippy::match_same_arms)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::Class(class) = node else {
            return;
        };

        // Classes that extend another class may rely on inheritance behavior.
        if class.super_class.is_some() {
            return;
        }

        let elements = &class.body;

        // Check if the class has a constructor or any instance member.
        if has_constructor_or_instance_member(elements, ctx) {
            return;
        }

        // At this point the class is either empty or all-static — flag it.
        let message = if elements.is_empty() {
            "Empty class is unnecessary — use an object literal or remove it"
        } else {
            "Class contains only static members — use a plain object or module-level functions instead"
        };

        ctx.report(Diagnostic {
            rule_name: "typescript/no-extraneous-class".to_owned(),
            message: message.to_owned(),
            span: Span::new(class.span.start, class.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }
}

/// Check whether a class body contains a constructor or any instance (non-static) member.
fn has_constructor_or_instance_member(elements: &[NodeId], ctx: &LintContext<'_>) -> bool {
    for element_id in elements {
        match ctx.node(*element_id) {
            Some(AstNode::MethodDefinition(method)) => {
                if method.kind == MethodDefinitionKind::Constructor {
                    return true;
                }
                if !method.is_static {
                    return true;
                }
            }
            Some(AstNode::PropertyDefinition(prop)) => {
                if !prop.is_static {
                    return true;
                }
            }
            // Static blocks are inherently static
            Some(AstNode::StaticBlock(_)) => {}
            // Anything else (unknown member types) we treat as instance-like
            // to avoid false positives.
            Some(_) => {
                return true;
            }
            None => {}
        }
    }
    false
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoExtraneousClass)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_static_only_class() {
        let diags = lint("class C { static foo() {} }");
        assert_eq!(
            diags.len(),
            1,
            "class with only static members should be flagged"
        );
    }

    #[test]
    fn test_flags_empty_class() {
        let diags = lint("class C {}");
        assert_eq!(diags.len(), 1, "empty class should be flagged");
    }

    #[test]
    fn test_allows_class_with_instance_method() {
        let diags = lint("class C { foo() {} }");
        assert!(
            diags.is_empty(),
            "class with instance method should not be flagged"
        );
    }

    #[test]
    fn test_allows_class_with_extends() {
        let diags = lint("class C extends Base {}");
        assert!(
            diags.is_empty(),
            "class extending a base class should not be flagged"
        );
    }

    #[test]
    fn test_allows_class_with_constructor() {
        let diags = lint("class C { constructor() {} }");
        assert!(
            diags.is_empty(),
            "class with a constructor should not be flagged"
        );
    }
}
