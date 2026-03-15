//! Rule: `react/jsx-pascal-case`
//!
//! Warn when user-defined JSX components don't use `PascalCase` naming.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::case_utils::is_pascal_case;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-pascal-case";

/// Flags user-defined JSX component names that are not `PascalCase`.
#[derive(Debug)]
pub struct JsxPascalCase;

impl LintRule for JsxPascalCase {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce PascalCase for user-defined JSX components".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXOpeningElement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXOpeningElement(opening) = node else {
            return;
        };

        // opening.name is a String directly
        let name = opening.name.as_str();

        // Only check user-defined components (start with uppercase)
        let Some(first) = name.chars().next() else {
            return;
        };
        if !first.is_ascii_uppercase() {
            return;
        }

        if !is_pascal_case(name) {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!("Component `{name}` should use PascalCase naming"),
                span: Span::new(opening.span.start, opening.span.end),
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

    starlint_rule_framework::lint_rule_test!(JsxPascalCase);

    #[test]
    fn test_flags_snake_case_component() {
        let diags = lint("const el = <My_Component />;");
        assert_eq!(diags.len(), 1, "should flag snake_case component name");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_pascal_case() {
        let diags = lint("const el = <MyComponent />;");
        assert!(diags.is_empty(), "should not flag PascalCase name");
    }

    #[test]
    fn test_allows_all_caps() {
        let diags = lint("const el = <SVG />;");
        assert!(diags.is_empty(), "should not flag ALL_CAPS name");
    }
}
