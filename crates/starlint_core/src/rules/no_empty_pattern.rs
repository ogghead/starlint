//! Rule: `no-empty-pattern`
//!
//! Disallow empty destructuring patterns. An empty pattern like `const {} = foo`
//! or `const [] = bar` looks like a destructuring assignment but doesn't
//! actually create any bindings. It almost always indicates a typo where the
//! developer meant to use a default value `{ a = {} }` instead of destructuring
//! `{ a: {} }`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags empty destructuring patterns (empty object `{}` or array `[]` patterns).
#[derive(Debug)]
pub struct NoEmptyPattern;

impl LintRule for NoEmptyPattern {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-empty-pattern".to_owned(),
            description: "Disallow empty destructuring patterns".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ArrayPattern, AstNodeType::ObjectPattern])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::ObjectPattern(pat) if pat.properties.is_empty() && pat.rest.is_none() => {
                ctx.report(Diagnostic {
                    rule_name: "no-empty-pattern".to_owned(),
                    message: "Unexpected empty object pattern".to_owned(),
                    span: Span::new(pat.span.start, pat.span.end),
                    severity: Severity::Error,
                    help: Some(
                        "If this is a default value, use `= {}` instead of `: {}`".to_owned(),
                    ),
                    fix: None,
                    labels: vec![],
                });
            }
            AstNode::ArrayPattern(pat) if pat.elements.is_empty() && pat.rest.is_none() => {
                ctx.report(Diagnostic {
                    rule_name: "no-empty-pattern".to_owned(),
                    message: "Unexpected empty array pattern".to_owned(),
                    span: Span::new(pat.span.start, pat.span.end),
                    severity: Severity::Error,
                    help: Some(
                        "If this is a default value, use `= []` instead of `: []`".to_owned(),
                    ),
                    fix: None,
                    labels: vec![],
                });
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoEmptyPattern)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_empty_object_pattern() {
        let diags = lint("const {} = foo;");
        assert_eq!(diags.len(), 1, "empty object pattern should be flagged");
    }

    #[test]
    fn test_flags_empty_array_pattern() {
        let diags = lint("const [] = foo;");
        assert_eq!(diags.len(), 1, "empty array pattern should be flagged");
    }

    #[test]
    fn test_flags_empty_pattern_in_params() {
        let diags = lint("function f({}) {}");
        assert_eq!(diags.len(), 1, "empty pattern in params should be flagged");
    }

    #[test]
    fn test_allows_non_empty_object_pattern() {
        let diags = lint("const { a } = foo;");
        assert!(
            diags.is_empty(),
            "non-empty object pattern should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_empty_array_pattern() {
        let diags = lint("const [a] = foo;");
        assert!(
            diags.is_empty(),
            "non-empty array pattern should not be flagged"
        );
    }

    #[test]
    fn test_allows_rest_element() {
        let diags = lint("const [...rest] = foo;");
        assert!(
            diags.is_empty(),
            "rest element in array pattern should not be flagged"
        );
    }

    #[test]
    fn test_allows_rest_in_object() {
        let diags = lint("const { ...rest } = foo;");
        assert!(
            diags.is_empty(),
            "rest in object pattern should not be flagged"
        );
    }
}
