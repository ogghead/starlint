//! Rule: `storybook/no-stories-of`
//!
//! `storiesOf` is deprecated and should not be used.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "storybook/no-stories-of";

/// `storiesOf` is deprecated and should not be used.
#[derive(Debug)]
pub struct NoStoriesOf;

impl LintRule for NoStoriesOf {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "`storiesOf` is deprecated and should not be used".to_owned(),
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

        let is_stories_of = matches!(
            ctx.node(call.callee),
            Some(AstNode::IdentifierReference(ident)) if ident.name.as_str() == "storiesOf"
        );

        if is_stories_of {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "`storiesOf` is deprecated — use CSF (Component Story Format) instead"
                    .to_owned(),
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

    starlint_rule_framework::lint_rule_test!(NoStoriesOf);

    #[test]
    fn test_flags_stories_of() {
        let diags = lint("storiesOf('Button', module).add('default', () => {});");
        assert_eq!(diags.len(), 1, "should flag storiesOf call");
    }

    #[test]
    fn test_allows_csf() {
        let diags = lint("export default { title: 'Button' }; export const Default = {};");
        assert!(diags.is_empty(), "should allow CSF format");
    }

    #[test]
    fn test_allows_other_calls() {
        let diags = lint("someFunction('Button');");
        assert!(diags.is_empty(), "should allow other function calls");
    }
}
