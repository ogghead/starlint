//! Rule: `no-unexpected-multiline`
//!
//! Disallow confusing multiline expressions where a newline looks like it is
//! ending a statement, but is not. For example, a function call that starts
//! on the next line without a semicolon:
//!
//! ```js
//! var foo = bar
//! (1 || 2).baz();
//! ```
//!
//! This rule flags cases where `(`, `[`, or a template literal follows a
//! newline after an expression statement.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags confusing multiline expressions that look like separate statements.
#[derive(Debug)]
pub struct NoUnexpectedMultiline;

impl LintRule for NoUnexpectedMultiline {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unexpected-multiline".to_owned(),
            description: "Disallow confusing multiline expressions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::CallExpression,
            AstNodeType::TaggedTemplateExpression,
        ])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::CallExpression(call) => {
                let callee_end = ctx.node(call.callee).map_or(0, |n| n.span().end);
                if callee_end > 0 {
                    let has_newline =
                        check_newline_before_paren(ctx.source_text(), callee_end, call.span.end);
                    if has_newline {
                        ctx.report(Diagnostic {
                            rule_name: "no-unexpected-multiline".to_owned(),
                            message:
                                "Unexpected newline between function name and opening parenthesis"
                                    .to_owned(),
                            span: Span::new(call.span.start, call.span.end),
                            severity: Severity::Error,
                            help: None,
                            fix: None,
                            labels: vec![],
                        });
                    }
                }
            }
            AstNode::TaggedTemplateExpression(tagged) => {
                let tag_end = ctx.node(tagged.tag).map_or(0, |n| n.span().end);
                if tag_end > 0 {
                    let template_start = ctx.node(tagged.quasi).map_or(0, |n| n.span().start);
                    let has_newline =
                        check_newline_between(ctx.source_text(), tag_end, template_start);
                    if has_newline {
                        ctx.report(Diagnostic {
                            rule_name: "no-unexpected-multiline".to_owned(),
                            message: "Unexpected newline between tag and template literal"
                                .to_owned(),
                            span: Span::new(tagged.span.start, tagged.span.end),
                            severity: Severity::Error,
                            help: None,
                            fix: None,
                            labels: vec![],
                        });
                    }
                }
            }
            _ => {}
        }
    }
}

/// Check if there is a newline before a `(` between two byte offsets.
fn check_newline_before_paren(source: &str, start: u32, end: u32) -> bool {
    let start_idx = usize::try_from(start).unwrap_or(0);
    let end_idx = usize::try_from(end).unwrap_or(0);
    let Some(between) = source.get(start_idx..end_idx) else {
        return false;
    };
    let Some(paren_pos) = between.find('(') else {
        return false;
    };
    let Some(before_paren) = between.get(..paren_pos) else {
        return false;
    };
    before_paren.contains('\n')
}

/// Check if there is a newline between two byte offsets.
fn check_newline_between(source: &str, start: u32, end: u32) -> bool {
    let start_idx = usize::try_from(start).unwrap_or(0);
    let end_idx = usize::try_from(end).unwrap_or(0);
    source
        .get(start_idx..end_idx)
        .is_some_and(|s| s.contains('\n'))
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUnexpectedMultiline)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_allows_same_line_call() {
        let diags = lint("var x = foo(1);");
        assert!(diags.is_empty(), "same-line call should not be flagged");
    }

    #[test]
    fn test_allows_semicolon_terminated() {
        let diags = lint("var x = foo;\n(1 || 2).baz();");
        assert!(
            diags.is_empty(),
            "semicolon-terminated line should not be flagged"
        );
    }

    #[test]
    fn test_allows_normal_code() {
        let diags = lint("var a = 1;\nvar b = 2;");
        assert!(diags.is_empty(), "normal code should not be flagged");
    }
}
