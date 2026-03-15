//! Rule: `prefer-array-flat`
//!
//! Prefer `Array.prototype.flat()` over legacy flattening patterns.
//! Flags `.reduce()` calls whose callback body contains `.concat()`,
//! which is a common pattern for flattening arrays before `.flat()` existed.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.reduce()` calls that likely flatten arrays using `.concat()`.
#[derive(Debug)]
pub struct PreferArrayFlat;

impl LintRule for PreferArrayFlat {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-array-flat".to_owned(),
            description: "Prefer `.flat()` over `.reduce()` with `.concat()`".to_owned(),
            category: Category::Style,
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

        // Must be a `.reduce()` call.
        let (member_span, prop_len) = {
            let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
                return;
            };
            if member.property.as_str() != "reduce" {
                return;
            }
            (member.span, member.property.len())
        };

        // Check source text of the call for `.concat(` — a simple heuristic
        // that catches the common `(a, b) => a.concat(b)` pattern without
        // deep AST inspection of the callback body.
        let start = usize::try_from(call.span.start).unwrap_or(0);
        let end = usize::try_from(call.span.end).unwrap_or(0);
        let Some(raw) = ctx.source_text().get(start..end) else {
            return;
        };

        if !raw.contains(".concat(") {
            return;
        }

        // Autofix: replace `reduce(...)` with `flat()` (from property name to end of call)
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        let prop_start = member_span.end.saturating_sub(prop_len as u32);
        ctx.report(Diagnostic {
            rule_name: "prefer-array-flat".to_owned(),
            message: "Prefer `.flat()` over `.reduce()` with `.concat()` for flattening arrays"
                .to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some("Replace with `.flat()`".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Replace with `.flat()`".to_owned(),
                edits: vec![Edit {
                    span: Span::new(prop_start, call.span.end),
                    replacement: "flat()".to_owned(),
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferArrayFlat);

    #[test]
    fn test_flags_reduce_concat() {
        let diags = lint("const flat = arr.reduce((a, b) => a.concat(b), []);");
        assert_eq!(diags.len(), 1, "should flag .reduce() with .concat()");
    }

    #[test]
    fn test_allows_reduce_without_concat() {
        let diags = lint("const sum = arr.reduce((a, b) => a + b, 0);");
        assert!(
            diags.is_empty(),
            ".reduce() without .concat() should not be flagged"
        );
    }

    #[test]
    fn test_allows_flat() {
        let diags = lint("const flat = arr.flat();");
        assert!(diags.is_empty(), ".flat() should not be flagged");
    }
}
