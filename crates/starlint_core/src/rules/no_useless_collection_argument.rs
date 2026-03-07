//! Rule: `no-useless-collection-argument`
//!
//! Flag unnecessary empty array arguments passed to `new Set()`, `new Map()`,
//! `new WeakSet()`, or `new WeakMap()`. Passing `[]` is equivalent to calling
//! the constructor with no arguments.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `new Set([])`, `new Map([])`, `new WeakSet([])`, and `new WeakMap([])`.
#[derive(Debug)]
pub struct NoUselessCollectionArgument;

/// Collection constructor names that accept an iterable.
const COLLECTION_TYPES: &[&str] = &["Set", "Map", "WeakSet", "WeakMap"];

impl LintRule for NoUselessCollectionArgument {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-collection-argument".to_owned(),
            description: "Disallow passing an empty array to collection constructors".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::NewExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::NewExpression(new_expr) = node else {
            return;
        };

        let Some(AstNode::IdentifierReference(id)) = ctx.node(new_expr.callee) else {
            return;
        };

        let name = id.name.as_str();
        if !COLLECTION_TYPES.contains(&name) {
            return;
        }

        // Check if first argument is an empty array literal `[]`
        let Some(first_arg_id) = new_expr.arguments.first() else {
            return;
        };

        let Some(AstNode::ArrayExpression(arr)) = ctx.node(*first_arg_id) else {
            return;
        };

        if !arr.elements.is_empty() {
            return;
        }

        let expr_span = Span::new(new_expr.span.start, new_expr.span.end);
        let arg_span = Span::new(arr.span.start, arr.span.end);
        let name_owned = name.to_owned();
        ctx.report(Diagnostic {
            rule_name: "no-useless-collection-argument".to_owned(),
            message: format!("Unnecessary empty array argument in `new {name_owned}([])` — use `new {name_owned}()` instead"),
            span: expr_span,
            severity: Severity::Warning,
            help: Some(format!("Use `new {name_owned}()` instead")),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Remove empty array argument".to_owned(),
                edits: vec![Edit {
                    span: arg_span,
                    replacement: String::new(),
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
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUselessCollectionArgument)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_new_set_empty_array() {
        let diags = lint("new Set([]);");
        assert_eq!(diags.len(), 1, "new Set([]) should be flagged");
    }

    #[test]
    fn test_flags_new_map_empty_array() {
        let diags = lint("new Map([]);");
        assert_eq!(diags.len(), 1, "new Map([]) should be flagged");
    }

    #[test]
    fn test_flags_new_weakset_empty_array() {
        let diags = lint("new WeakSet([]);");
        assert_eq!(diags.len(), 1, "new WeakSet([]) should be flagged");
    }

    #[test]
    fn test_flags_new_weakmap_empty_array() {
        let diags = lint("new WeakMap([]);");
        assert_eq!(diags.len(), 1, "new WeakMap([]) should be flagged");
    }

    #[test]
    fn test_allows_no_argument() {
        let diags = lint("new Set();");
        assert!(diags.is_empty(), "new Set() should not be flagged");
    }

    #[test]
    fn test_allows_non_empty_array() {
        let diags = lint("new Set([1, 2]);");
        assert!(diags.is_empty(), "new Set([1, 2]) should not be flagged");
    }

    #[test]
    fn test_allows_variable_argument() {
        let diags = lint("new Set(items);");
        assert!(diags.is_empty(), "new Set(items) should not be flagged");
    }

    #[test]
    fn test_allows_non_collection_constructor() {
        let diags = lint("new Array([]);");
        assert!(
            diags.is_empty(),
            "new Array([]) should not be flagged by this rule"
        );
    }
}
