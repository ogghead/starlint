//! Rule: `no-static-only-class`
//!
//! Disallow classes that contain only static members. A class with exclusively
//! static methods and properties is better expressed as a plain object or
//! module-level exports -- the `class` keyword adds no value when there is no
//! instantiation or inheritance.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags classes whose body consists entirely of static members.
#[derive(Debug)]
pub struct NoStaticOnlyClass;

impl LintRule for NoStaticOnlyClass {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-static-only-class".to_owned(),
            description: "Disallow classes with only static members".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::Class])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::Class(class) = node else {
            return;
        };

        // Skip classes with a superclass -- they may rely on inheritance.
        if class.super_class.is_some() {
            return;
        }

        let elements = &class.body;

        // Empty classes are not flagged.
        if elements.is_empty() {
            return;
        }

        let all_static = elements.iter().all(|element_id| {
            match ctx.node(*element_id) {
                Some(AstNode::MethodDefinition(method)) => method.is_static,
                Some(AstNode::PropertyDefinition(prop)) => prop.is_static,
                // Static blocks are inherently static.
                Some(AstNode::StaticBlock(_)) => true,
                _ => false,
            }
        });

        if all_static {
            ctx.report(Diagnostic {
                rule_name: "no-static-only-class".to_owned(),
                message: "Class contains only static members -- use a plain object or module exports instead".to_owned(),
                span: Span::new(class.span.start, class.span.end),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoStaticOnlyClass)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_all_static_members() {
        let diags = lint("class Foo { static bar() {} static baz = 1; }");
        assert_eq!(
            diags.len(),
            1,
            "class with only static members should be flagged"
        );
    }

    #[test]
    fn test_flags_single_static_method() {
        let diags = lint("class Foo { static bar() {} }");
        assert_eq!(
            diags.len(),
            1,
            "class with a single static method should be flagged"
        );
    }

    #[test]
    fn test_allows_instance_method() {
        let diags = lint("class Foo { bar() {} }");
        assert!(
            diags.is_empty(),
            "class with instance method should not be flagged"
        );
    }

    #[test]
    fn test_allows_mixed_static_and_instance() {
        let diags = lint("class Foo { static bar() {} baz() {} }");
        assert!(
            diags.is_empty(),
            "class with mixed members should not be flagged"
        );
    }

    #[test]
    fn test_allows_empty_class() {
        let diags = lint("class Foo {}");
        assert!(diags.is_empty(), "empty class should not be flagged");
    }

    #[test]
    fn test_allows_class_with_superclass() {
        let diags = lint("class Foo extends Base { static bar() {} }");
        assert!(
            diags.is_empty(),
            "class extending a superclass should not be flagged"
        );
    }

    #[test]
    fn test_allows_static_and_instance_property() {
        let diags = lint("class Foo { static x = 1; y = 2; }");
        assert!(
            diags.is_empty(),
            "class with static and instance properties should not be flagged"
        );
    }
}
