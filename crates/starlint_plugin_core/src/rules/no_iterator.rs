//! Rule: `no-iterator`
//!
//! Disallow the use of the `__iterator__` property. This is an obsolete
//! SpiderMonkey-specific extension. Use `Symbol.iterator` instead.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags usage of the `__iterator__` property.
#[derive(Debug)]
pub struct NoIterator;

impl LintRule for NoIterator {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-iterator".to_owned(),
            description: "Disallow the `__iterator__` property".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::StaticMemberExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::StaticMemberExpression(member) = node else {
            return;
        };

        if member.property.as_str() == "__iterator__" {
            // Fix: obj.__iterator__ → obj[Symbol.iterator]
            #[allow(clippy::as_conversions)]
            let fix = ctx
                .node(member.object)
                .and_then(|obj_node| {
                    let obj_span = obj_node.span();
                    ctx.source_text()
                        .get(obj_span.start as usize..obj_span.end as usize)
                })
                .map(|obj_text| {
                    let replacement = format!("{obj_text}[Symbol.iterator]");
                    Fix {
                        kind: FixKind::SuggestionFix,
                        message: format!("Replace with `{replacement}`"),
                        edits: vec![Edit {
                            span: Span::new(member.span.start, member.span.end),
                            replacement,
                        }],
                        is_snippet: false,
                    }
                });

            ctx.report(Diagnostic {
                rule_name: "no-iterator".to_owned(),
                message: "Use `Symbol.iterator` instead of `__iterator__`".to_owned(),
                span: Span::new(member.span.start, member.span.end),
                severity: Severity::Warning,
                help: Some("Replace `.__iterator__` with `[Symbol.iterator]`".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    starlint_rule_framework::lint_rule_test!(NoIterator);

    #[test]
    fn test_flags_iterator_property() {
        let diags = lint("foo.__iterator__ = function() {};");
        assert_eq!(diags.len(), 1, "__iterator__ property should be flagged");
    }

    #[test]
    fn test_allows_symbol_iterator() {
        let diags = lint("foo[Symbol.iterator] = function() {};");
        assert!(diags.is_empty(), "Symbol.iterator should not be flagged");
    }

    #[test]
    fn test_allows_normal_property() {
        let diags = lint("foo.bar = 1;");
        assert!(diags.is_empty(), "normal property should not be flagged");
    }
}
