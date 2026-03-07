//! Rule: `react/no-children-prop`
//!
//! Warn when passing `children` as a prop rather than nesting children inside the element.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags usage of `children` as a JSX prop.
#[derive(Debug)]
pub struct NoChildrenProp;

impl LintRule for NoChildrenProp {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-children-prop".to_owned(),
            description: "Disallow passing `children` as a prop".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXAttribute])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXAttribute(attr) = node else {
            return;
        };

        // In starlint_ast, attr.name is a String
        let name = attr.name.as_str();
        // Skip namespaced names
        if name.contains(':') {
            return;
        }

        if name == "children" {
            let attr_span = Span::new(attr.span.start, attr.span.end);
            let fix = FixBuilder::new("Remove `children` prop", FixKind::SuggestionFix)
                .edit(fix_utils::remove_jsx_attr(ctx.source_text(), attr_span))
                .build();
            ctx.report(Diagnostic {
                rule_name: "react/no-children-prop".to_owned(),
                message: "Do not pass `children` as a prop — nest children between opening and closing tags instead".to_owned(),
                span: attr_span,
                severity: Severity::Warning,
                help: None,
                fix,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoChildrenProp)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_children_prop() {
        let diags = lint(r#"const x = <div children="hello" />;"#);
        assert_eq!(diags.len(), 1, "should flag children prop");
    }

    #[test]
    fn test_allows_nested_children() {
        let diags = lint(r"const x = <div>hello</div>;");
        assert!(diags.is_empty(), "nested children should not be flagged");
    }

    #[test]
    fn test_flags_children_expression() {
        let diags = lint(r"const x = <Comp children={<span />} />;");
        assert_eq!(
            diags.len(),
            1,
            "should flag children prop with expression value"
        );
    }
}
