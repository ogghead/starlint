//! Rule: `node/no-path-concat`
//!
//! Disallow string concatenation with `__dirname` or `__filename`.
//! Building file paths with `+` is fragile and platform-dependent.
//! Use `path.join()` or `path.resolve()` instead.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags string concatenation (`+`) involving `__dirname` or `__filename`.
#[derive(Debug)]
pub struct NoPathConcat;

/// Check whether a node is an identifier named `__dirname` or `__filename`.
fn is_path_global(node: &AstNode) -> bool {
    matches!(
        node,
        AstNode::IdentifierReference(id)
            if id.name.as_str() == "__dirname" || id.name.as_str() == "__filename"
    )
}

impl LintRule for NoPathConcat {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "node/no-path-concat".to_owned(),
            description: "Disallow string concatenation with `__dirname` or `__filename`"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BinaryExpression])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::BinaryExpression(expr) = node else {
            return;
        };

        if expr.operator != BinaryOperator::Addition {
            return;
        }

        let left_node = ctx.node(expr.left);
        let right_node = ctx.node(expr.right);
        let is_left_path = left_node.is_some_and(is_path_global);
        let is_right_path = right_node.is_some_and(is_path_global);
        if !is_left_path && !is_right_path {
            return;
        }

        // Build suggestion fix: __dirname + '/foo' → path.join(__dirname, '/foo')
        let source = ctx.source_text();
        let left_span = left_node.map_or(Span::new(0, 0), |n| {
            let s = n.span();
            Span::new(s.start, s.end)
        });
        let right_span = right_node.map_or(Span::new(0, 0), |n| {
            let s = n.span();
            Span::new(s.start, s.end)
        });
        let left_text = source
            .get(left_span.start as usize..left_span.end as usize)
            .unwrap_or("")
            .to_owned();
        let right_text = source
            .get(right_span.start as usize..right_span.end as usize)
            .unwrap_or("")
            .to_owned();
        let fix = (!left_text.is_empty() && !right_text.is_empty()).then(|| Fix {
            kind: FixKind::SuggestionFix,
            message: format!("Replace with `path.join({left_text}, {right_text})`"),
            edits: vec![Edit {
                span: Span::new(expr.span.start, expr.span.end),
                replacement: format!("path.join({left_text}, {right_text})"),
            }],
            is_snippet: false,
        });

        ctx.report(Diagnostic {
            rule_name: "node/no-path-concat".to_owned(),
            message: "Do not concatenate paths with `+` \u{2014} use `path.join()` or `path.resolve()` instead".to_owned(),
            span: Span::new(expr.span.start, expr.span.end),
            severity: Severity::Warning,
            help: Some("Use `path.join()` or `path.resolve()` instead".to_owned()),
            fix,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoPathConcat);

    #[test]
    fn test_flags_dirname_concat() {
        let diags = lint("var p = __dirname + '/foo';");
        assert_eq!(diags.len(), 1, "__dirname + string should be flagged");
    }

    #[test]
    fn test_flags_filename_concat() {
        let diags = lint("var p = '/bar' + __filename;");
        assert_eq!(diags.len(), 1, "string + __filename should be flagged");
    }

    #[test]
    fn test_allows_path_join() {
        let diags = lint("var p = path.join(__dirname, 'foo');");
        assert!(diags.is_empty(), "path.join should not be flagged");
    }

    #[test]
    fn test_allows_normal_concat() {
        let diags = lint("var s = a + b;");
        assert!(
            diags.is_empty(),
            "normal concatenation should not be flagged"
        );
    }

    #[test]
    fn test_allows_string_addition() {
        let diags = lint("var s = 'hello' + 'world';");
        assert!(
            diags.is_empty(),
            "normal string addition should not be flagged"
        );
    }
}
