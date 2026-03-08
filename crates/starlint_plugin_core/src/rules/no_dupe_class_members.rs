//! Rule: `no-dupe-class-members`
//!
//! Disallow duplicate names in class members. Having two methods or properties
//! with the same name in a class body means the second one silently overwrites
//! the first, which is almost always a mistake.

use std::collections::HashSet;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::MethodDefinitionKind;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags duplicate method/property names in class bodies.
#[derive(Debug)]
pub struct NoDupeClassMembers;

impl LintRule for NoDupeClassMembers {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-dupe-class-members".to_owned(),
            description: "Disallow duplicate class members".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::Class])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::Class(class) = node else {
            return;
        };

        // Track seen member keys as (is_static, name) pairs
        let mut seen: HashSet<(bool, String)> = HashSet::new();

        for element_id in &class.body {
            let Some(AstNode::MethodDefinition(method)) = ctx.node(*element_id) else {
                continue;
            };

            // Skip getters and setters — a getter/setter pair with the
            // same name is valid.
            if method.kind == MethodDefinitionKind::Get || method.kind == MethodDefinitionKind::Set
            {
                continue;
            }

            // Skip computed properties — we can't statically determine duplicates.
            if method.computed {
                continue;
            }

            let Some(name) = static_key_name(method.key, ctx) else {
                continue;
            };

            let method_span = method.span;
            let is_static = method.is_static;
            let key = (is_static, name.clone());
            if !seen.insert(key) {
                ctx.report(Diagnostic {
                    rule_name: "no-dupe-class-members".to_owned(),
                    message: format!("Duplicate class member `{name}`"),
                    span: Span::new(method_span.start, method_span.end),
                    severity: Severity::Error,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
        }
    }
}

/// Extract the name of a non-computed property key.
fn static_key_name(key_id: NodeId, ctx: &LintContext<'_>) -> Option<String> {
    match ctx.node(key_id)? {
        AstNode::IdentifierReference(ident) => Some(ident.name.clone()),
        AstNode::StringLiteral(lit) => Some(lit.value.clone()),
        AstNode::NumericLiteral(lit) => Some(lit.value.to_string()),
        AstNode::BindingIdentifier(ident) => Some(ident.name.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoDupeClassMembers)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_duplicate_methods() {
        let diags = lint("class Foo { bar() {} bar() {} }");
        assert_eq!(diags.len(), 1, "duplicate method should be flagged");
    }

    #[test]
    fn test_allows_different_methods() {
        let diags = lint("class Foo { bar() {} baz() {} }");
        assert!(diags.is_empty(), "different methods should not be flagged");
    }

    #[test]
    fn test_allows_getter_setter_pair() {
        let diags = lint("class Foo { get bar() {} set bar(v) {} }");
        assert!(diags.is_empty(), "getter/setter pair should not be flagged");
    }

    #[test]
    fn test_flags_duplicate_static_methods() {
        let diags = lint("class Foo { static bar() {} static bar() {} }");
        assert_eq!(diags.len(), 1, "duplicate static methods should be flagged");
    }

    #[test]
    fn test_allows_static_and_instance_same_name() {
        let diags = lint("class Foo { bar() {} static bar() {} }");
        assert!(
            diags.is_empty(),
            "static and instance with same name should not be flagged"
        );
    }

    #[test]
    fn test_allows_constructor() {
        let diags = lint("class Foo { constructor() {} bar() {} }");
        assert!(
            diags.is_empty(),
            "constructor with other methods should not be flagged"
        );
    }
}
