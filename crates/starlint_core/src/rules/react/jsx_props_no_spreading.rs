//! Rule: `react/jsx-props-no-spreading`
//!
//! Warn against using spread attributes `{...props}` in JSX.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-props-no-spreading";

/// Flags JSX spread attributes (`{...props}`). Spreading makes it harder to
/// track which props a component receives and can pass unintended props.
#[derive(Debug)]
pub struct JsxPropsNoSpreading;

impl LintRule for JsxPropsNoSpreading {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow spreading props in JSX".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXSpreadAttribute])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXSpreadAttribute(spread) = node else {
            return;
        };

        let spread_span = Span::new(spread.span.start, spread.span.end);
        let fix = FixBuilder::new("Remove spread attribute", FixKind::SuggestionFix)
            .edit(fix_utils::remove_jsx_attr(ctx.source_text(), spread_span))
            .build();
        ctx.report(Diagnostic {
            rule_name: RULE_NAME.to_owned(),
            message: "Avoid spreading props in JSX — pass props explicitly for clarity".to_owned(),
            span: spread_span,
            severity: Severity::Warning,
            help: None,
            fix,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(JsxPropsNoSpreading)];
        lint_source(source, "test.tsx", &rules)
    }

    #[test]
    fn test_flags_spread_props() {
        let diags = lint("const el = <div {...props} />;");
        assert_eq!(diags.len(), 1, "should flag spread attributes");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_explicit_props() {
        let diags = lint(r#"const el = <div className="foo" id="bar" />;"#);
        assert!(diags.is_empty(), "should not flag explicit props");
    }

    #[test]
    fn test_flags_multiple_spreads() {
        let diags = lint("const el = <div {...a} {...b} />;");
        assert_eq!(diags.len(), 2, "should flag each spread attribute");
    }
}
