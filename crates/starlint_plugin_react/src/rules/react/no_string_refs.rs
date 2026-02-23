//! Rule: `react/no-string-refs`
//!
//! Warn when string refs are used (`ref="myRef"`). String refs are deprecated.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::fix_utils;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags usage of string refs like `ref="myRef"`.
#[derive(Debug)]
pub struct NoStringRefs;

impl LintRule for NoStringRefs {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-string-refs".to_owned(),
            description: "Disallow using string refs (deprecated)".to_owned(),
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

        // JSXAttributeNode.name is a String directly in starlint_ast
        let name = attr.name.as_str();

        if name != "ref" {
            return;
        }

        // Only flag when the value is a string literal
        let is_string_value = attr
            .value
            .and_then(|v| ctx.node(v))
            .is_some_and(|n| matches!(n, AstNode::StringLiteral(_)));
        if is_string_value {
            let attr_span = Span::new(attr.span.start, attr.span.end);
            let fix = FixBuilder::new("Remove string ref", FixKind::SuggestionFix)
                .edit(fix_utils::remove_jsx_attr(ctx.source_text(), attr_span))
                .build();
            ctx.report(Diagnostic {
                rule_name: "react/no-string-refs".to_owned(),
                message: "String refs are deprecated — use `useRef` or callback refs instead"
                    .to_owned(),
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
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoStringRefs)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_string_ref() {
        let diags = lint(r#"const x = <div ref="myRef" />;"#);
        assert_eq!(diags.len(), 1, "should flag string ref");
    }

    #[test]
    fn test_allows_callback_ref() {
        let diags = lint(r"const x = <div ref={myRef} />;");
        assert!(diags.is_empty(), "callback ref should not be flagged");
    }

    #[test]
    fn test_allows_non_ref_prop() {
        let diags = lint(r#"const x = <div id="myDiv" />;"#);
        assert!(diags.is_empty(), "non-ref props should not be flagged");
    }
}
