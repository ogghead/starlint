//! Rule: `react/no-redundant-should-component-update`
//!
//! Flag `shouldComponentUpdate` when extending `PureComponent`. `PureComponent`
//! already implements a shallow comparison in `shouldComponentUpdate`, so
//! defining it again is redundant and likely a mistake.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `shouldComponentUpdate` in classes extending `PureComponent`.
#[derive(Debug)]
pub struct NoRedundantShouldComponentUpdate;

impl LintRule for NoRedundantShouldComponentUpdate {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-redundant-should-component-update".to_owned(),
            description: "Disallow `shouldComponentUpdate` when extending `PureComponent`"
                .to_owned(),
            category: Category::Correctness,
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

        // Check if the class extends PureComponent or React.PureComponent
        if !extends_pure_component(class, ctx) {
            return;
        }

        for &element_id in &*class.body {
            let Some(AstNode::MethodDefinition(method)) = ctx.node(element_id) else {
                continue;
            };

            // Get method name from key
            let method_name = match ctx.node(method.key) {
                Some(AstNode::IdentifierReference(ident)) => ident.name.as_str(),
                Some(AstNode::BindingIdentifier(ident)) => ident.name.as_str(),
                Some(AstNode::StringLiteral(s)) => s.value.as_str(),
                _ => {
                    // Try source text as fallback
                    let key_span = ctx.node(method.key).map_or(
                        starlint_ast::types::Span::EMPTY,
                        starlint_ast::AstNode::span,
                    );

                    ctx.source_text()
                        .get(
                            usize::try_from(key_span.start).unwrap_or(0)
                                ..usize::try_from(key_span.end).unwrap_or(0),
                        )
                        .unwrap_or("")
                }
            };

            if method_name == "shouldComponentUpdate" {
                ctx.report(Diagnostic {
                    rule_name: "react/no-redundant-should-component-update".to_owned(),
                    message: "`shouldComponentUpdate` is redundant when extending `PureComponent`"
                        .to_owned(),
                    span: Span::new(method.span.start, method.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: Some(Fix {
                        kind: FixKind::SuggestionFix,
                        message: "Remove redundant `shouldComponentUpdate` method".to_owned(),
                        edits: vec![Edit {
                            span: Span::new(method.span.start, method.span.end),
                            replacement: String::new(),
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
            }
        }
    }
}

/// Check whether a class extends `PureComponent` or `React.PureComponent`.
fn extends_pure_component(class: &starlint_ast::node::ClassNode, ctx: &LintContext<'_>) -> bool {
    let Some(super_class_id) = class.super_class else {
        return false;
    };

    match ctx.node(super_class_id) {
        // class Foo extends PureComponent
        Some(AstNode::IdentifierReference(ident)) => ident.name.as_str() == "PureComponent",
        // class Foo extends React.PureComponent
        Some(AstNode::StaticMemberExpression(member)) => {
            member.property.as_str() == "PureComponent"
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoRedundantShouldComponentUpdate);

    #[test]
    fn test_flags_should_component_update_in_pure_component() {
        let source = r"
class MyComponent extends React.PureComponent {
    shouldComponentUpdate() {
        return true;
    }
}";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "shouldComponentUpdate in PureComponent should be flagged"
        );
    }

    #[test]
    fn test_flags_bare_pure_component() {
        let source = r"
class MyComponent extends PureComponent {
    shouldComponentUpdate() {
        return true;
    }
}";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "shouldComponentUpdate in bare PureComponent should be flagged"
        );
    }

    #[test]
    fn test_allows_in_regular_component() {
        let source = r"
class MyComponent extends React.Component {
    shouldComponentUpdate() {
        return true;
    }
}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "shouldComponentUpdate in Component should not be flagged"
        );
    }

    #[test]
    fn test_allows_pure_component_without_should_update() {
        let source = r"
class MyComponent extends React.PureComponent {
    render() {
        return null;
    }
}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "PureComponent without shouldComponentUpdate should not be flagged"
        );
    }
}
