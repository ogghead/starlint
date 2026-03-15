//! Rule: `typescript/no-for-in-array`
//!
//! Disallow iterating over arrays with `for...in`. The `for...in` statement
//! iterates over enumerable property *names* (string keys), not values.
//! When used on an array, the loop variable receives string indices (`"0"`,
//! `"1"`, ...) and may also include inherited enumerable properties. Use
//! `for...of` to iterate over array values instead.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags all `for...in` statements as potentially incorrect for array iteration.
#[derive(Debug)]
pub struct NoForInArray;

impl LintRule for NoForInArray {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-for-in-array".to_owned(),
            description: "Disallow iterating over arrays with `for...in`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ForInStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ForInStatement(stmt) = node else {
            return;
        };

        // Fix: for (x in arr) → for (x of arr)
        // stmt.left and stmt.right are NodeId — resolve their spans
        #[allow(clippy::as_conversions)]
        let fix = {
            let source = ctx.source_text();
            let left_span = ctx.node(stmt.left).map(AstNode::span);
            let right_span = ctx.node(stmt.right).map(AstNode::span);
            left_span.zip(right_span).and_then(|(ls, rs)| {
                let left_end = ls.end as usize;
                let right_start = rs.start as usize;
                let between = source.get(left_end..right_start).unwrap_or("");
                between.find(" in ").and_then(|pos| {
                    let in_start =
                        u32::try_from(left_end.saturating_add(pos).saturating_add(1)).ok()?;
                    let in_end = in_start.saturating_add(2);
                    Some(Fix {
                        kind: FixKind::SuggestionFix,
                        message: "Replace `in` with `of`".to_owned(),
                        edits: vec![Edit {
                            span: Span::new(in_start, in_end),
                            replacement: "of".to_owned(),
                        }],
                        is_snippet: false,
                    })
                })
            })
        };

        ctx.report(Diagnostic {
            rule_name: "typescript/no-for-in-array".to_owned(),
            message: "`for...in` iterates over string keys, not values — use `for...of` instead"
                .to_owned(),
            span: Span::new(stmt.span.start, stmt.span.end),
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

    starlint_rule_framework::lint_rule_test!(NoForInArray, "test.ts");

    #[test]
    fn test_flags_for_in_with_array() {
        let diags = lint("const arr = [1, 2, 3]; for (const key in arr) { console.log(key); }");
        assert_eq!(diags.len(), 1, "for-in on array should be flagged");
    }

    #[test]
    fn test_flags_for_in_with_variable() {
        let diags = lint("for (const k in someVar) {}");
        assert_eq!(diags.len(), 1, "for-in should be flagged");
    }

    #[test]
    fn test_flags_for_in_with_let() {
        let diags = lint("for (let key in obj) { use(key); }");
        assert_eq!(diags.len(), 1, "for-in with let should be flagged");
    }

    #[test]
    fn test_allows_for_of() {
        let diags = lint("for (const val of arr) { console.log(val); }");
        assert!(diags.is_empty(), "for-of should not be flagged");
    }

    #[test]
    fn test_allows_regular_for() {
        let diags = lint("for (let i = 0; i < 10; i++) {}");
        assert!(diags.is_empty(), "regular for loop should not be flagged");
    }
}
