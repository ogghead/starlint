//! Rule: `prefer-set-has`
//!
//! Prefer `Set#has()` over `Array#includes()` when checking membership in
//! an array literal. Array literals used as lookup tables should be converted
//! to a `Set` for O(1) lookups instead of O(n) scans.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.includes()` calls on array literals.
#[derive(Debug)]
pub struct PreferSetHas;

impl LintRule for PreferSetHas {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-set-has".to_owned(),
            description: "Prefer `Set#has()` over `Array#includes()` for array literal lookups"
                .to_owned(),
            category: Category::Suggestion,
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

        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property.as_str() != "includes" {
            return;
        }

        // Must have exactly one argument (the search value)
        if call.arguments.len() != 1 {
            return;
        }

        // The first argument must not be a spread element
        if let Some(&first_id) = call.arguments.first() {
            if matches!(ctx.node(first_id), Some(AstNode::SpreadElement(_))) {
                return;
            }
        }

        // The object must be an array literal
        if !matches!(ctx.node(member.object), Some(AstNode::ArrayExpression(_))) {
            return;
        }

        // Fix: [1,2,3].includes(x) -> new Set([1,2,3]).has(x)
        #[allow(clippy::as_conversions)]
        let fix = {
            let source = ctx.source_text();
            let arr_span = ctx.node(member.object).map_or(
                starlint_ast::types::Span::EMPTY,
                starlint_ast::AstNode::span,
            );
            let arr_text = source
                .get(arr_span.start as usize..arr_span.end as usize)
                .unwrap_or("");
            let arg = call.arguments.first().and_then(|&id| ctx.node(id));
            arg.and_then(|a| {
                let a_span = a.span();
                let a_text = source.get(a_span.start as usize..a_span.end as usize)?;
                let replacement = format!("new Set({arr_text}).has({a_text})");
                Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: format!("Replace with `{replacement}`"),
                    edits: vec![Edit {
                        span: Span::new(call.span.start, call.span.end),
                        replacement,
                    }],
                    is_snippet: false,
                })
            })
        };

        ctx.report(Diagnostic {
            rule_name: "prefer-set-has".to_owned(),
            message:
                "Use `new Set([...]).has()` instead of `[...].includes()` for better performance"
                    .to_owned(),
            span: Span::new(call.span.start, call.span.end),
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

    starlint_rule_framework::lint_rule_test!(PreferSetHas);

    #[test]
    fn test_flags_array_literal_includes() {
        let diags = lint("['a', 'b', 'c'].includes(x);");
        assert_eq!(
            diags.len(),
            1,
            "array literal .includes() should be flagged"
        );
    }

    #[test]
    fn test_flags_numeric_array_includes() {
        let diags = lint("[1, 2, 3].includes(val);");
        assert_eq!(
            diags.len(),
            1,
            "numeric array literal .includes() should be flagged"
        );
    }

    #[test]
    fn test_allows_variable_includes() {
        let diags = lint("arr.includes(x);");
        assert!(
            diags.is_empty(),
            "variable .includes() should not be flagged"
        );
    }

    #[test]
    fn test_allows_string_includes() {
        let diags = lint("str.includes('sub');");
        assert!(diags.is_empty(), "string .includes() should not be flagged");
    }

    #[test]
    fn test_allows_set_has() {
        let diags = lint("new Set(['a']).has(x);");
        assert!(diags.is_empty(), "Set.has() should not be flagged");
    }

    #[test]
    fn test_allows_includes_with_from_index() {
        let diags = lint("['a', 'b'].includes(x, 1);");
        assert!(
            diags.is_empty(),
            ".includes() with fromIndex should not be flagged"
        );
    }
}
