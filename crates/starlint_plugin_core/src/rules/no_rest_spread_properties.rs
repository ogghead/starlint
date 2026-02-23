//! Rule: `no-rest-spread-properties`
//!
//! Flag use of object rest/spread properties (`{...obj}` and
//! `const {a, ...rest} = obj`). Some codebases prefer avoiding these
//! for compatibility or clarity.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags object spread (`{...obj}`) and object rest (`const {a, ...rest} = obj`).
#[derive(Debug)]
pub struct NoRestSpreadProperties;

impl LintRule for NoRestSpreadProperties {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-rest-spread-properties".to_owned(),
            description: "Disallow object rest/spread properties".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ObjectExpression, AstNodeType::ObjectPattern])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::ObjectExpression(obj) => {
                for prop_id in &obj.properties {
                    if let Some(AstNode::SpreadElement(spread)) = ctx.node(*prop_id) {
                        ctx.report(Diagnostic {
                            rule_name: "no-rest-spread-properties".to_owned(),
                            message: "Unexpected object spread property".to_owned(),
                            span: Span::new(spread.span.start, spread.span.end),
                            severity: Severity::Warning,
                            help: None,
                            fix: None,
                            labels: vec![],
                        });
                    }
                }
            }
            AstNode::ObjectPattern(pat) => {
                if let Some(rest_id) = pat.rest {
                    let rest_ast_span = ctx.node(rest_id).map_or(
                        starlint_ast::types::Span::new(0, 0),
                        starlint_ast::AstNode::span,
                    );
                    ctx.report(Diagnostic {
                        rule_name: "no-rest-spread-properties".to_owned(),
                        message: "Unexpected object rest property".to_owned(),
                        span: Span::new(rest_ast_span.start, rest_ast_span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoRestSpreadProperties)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_object_spread() {
        let diags = lint("const x = {...obj};");
        assert_eq!(diags.len(), 1, "object spread should be flagged");
    }

    #[test]
    fn test_flags_object_rest() {
        let diags = lint("const {a, ...rest} = obj;");
        assert_eq!(diags.len(), 1, "object rest should be flagged");
    }

    #[test]
    fn test_allows_array_spread() {
        let diags = lint("const x = [1, 2, 3];");
        assert!(diags.is_empty(), "array literal should not be flagged");
    }

    #[test]
    fn test_allows_array_rest() {
        let diags = lint("const [a, ...rest] = arr;");
        assert!(diags.is_empty(), "array rest should not be flagged");
    }

    #[test]
    fn test_allows_plain_object() {
        let diags = lint("const x = { a: 1, b: 2 };");
        assert!(diags.is_empty(), "plain object should not be flagged");
    }

    #[test]
    fn test_flags_multiple_spreads() {
        let diags = lint("const x = {...a, ...b};");
        assert_eq!(
            diags.len(),
            2,
            "two spread properties should produce two diagnostics"
        );
    }
}
