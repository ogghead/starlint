//! Rule: `default-param-last`
//!
//! Enforce default parameters to be last. Non-default parameters after
//! a default parameter cannot take advantage of defaults without passing
//! `undefined` explicitly.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags default parameters that are not in the last positions.
#[derive(Debug)]
pub struct DefaultParamLast;

impl LintRule for DefaultParamLast {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "default-param-last".to_owned(),
            description: "Enforce default parameters to be last".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ArrowFunctionExpression, AstNodeType::Function])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    #[allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let (params, func_span) = match node {
            AstNode::Function(f) => (&f.params, f.span),
            AstNode::ArrowFunctionExpression(arrow) => (&arrow.params, arrow.span),
            _ => return,
        };

        if params.is_empty() {
            return;
        }

        // For each param, determine if it has a default value by checking the
        // source text between consecutive param spans for `=`.
        let source = ctx.source_text();
        // Collect resolved param spans
        let param_spans: Vec<_> = params
            .iter()
            .filter_map(|pid| ctx.node(*pid).map(starlint_ast::AstNode::span))
            .collect();

        if param_spans.is_empty() {
            return;
        }

        // Determine if each param has a default by looking at source text
        // between this param's end and the next param's start (or closing paren).
        let mut has_default = Vec::with_capacity(param_spans.len());
        for (i, ps) in param_spans.iter().enumerate() {
            let region_end = if i + 1 < param_spans.len() {
                param_spans[i + 1].start as usize
            } else {
                // Find the closing `)` after the last param
                func_span.end as usize
            };
            let region = source.get(ps.end as usize..region_end).unwrap_or("");
            has_default.push(region.contains('='));
        }

        // Find the last non-default, non-rest parameter.
        // Any default parameter before it is a violation.
        let mut seen_non_default = false;
        for (i, is_default) in has_default.iter().enumerate().rev() {
            if *is_default {
                if seen_non_default {
                    let ps = param_spans[i];
                    ctx.report(Diagnostic {
                        rule_name: "default-param-last".to_owned(),
                        message: "Default parameters should be last".to_owned(),
                        span: Span::new(ps.start, ps.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            } else {
                seen_non_default = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(DefaultParamLast)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_allows_defaults_at_end() {
        let diags = lint("function foo(a, b = 1) {}");
        assert!(diags.is_empty(), "default at end should not be flagged");
    }

    #[test]
    fn test_flags_default_before_non_default() {
        let diags = lint("function foo(a = 1, b) {}");
        assert_eq!(
            diags.len(),
            1,
            "default before non-default should be flagged"
        );
    }

    #[test]
    fn test_allows_all_defaults() {
        let diags = lint("function foo(a = 1, b = 2) {}");
        assert!(diags.is_empty(), "all defaults should not be flagged");
    }

    #[test]
    fn test_allows_no_defaults() {
        let diags = lint("function foo(a, b) {}");
        assert!(diags.is_empty(), "no defaults should not be flagged");
    }

    #[test]
    fn test_flags_arrow_function() {
        let diags = lint("const foo = (a = 1, b) => {};");
        assert_eq!(
            diags.len(),
            1,
            "arrow with default before non-default should be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_violations() {
        let diags = lint("function foo(a = 1, b = 2, c) {}");
        assert_eq!(
            diags.len(),
            2,
            "multiple defaults before non-default should all be flagged"
        );
    }
}
