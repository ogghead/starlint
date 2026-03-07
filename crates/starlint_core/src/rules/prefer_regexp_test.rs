//! Rule: `prefer-regexp-test` (unicorn)
//!
//! Prefer `RegExp#test()` over `String#match()` when only checking for
//! existence of a match. `test()` is faster and more semantically correct
//! when you don't need the matched value.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `string.match(regex)` that could use `regex.test(string)`.
#[derive(Debug)]
pub struct PreferRegexpTest;

impl LintRule for PreferRegexpTest {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-regexp-test".to_owned(),
            description: "Prefer RegExp#test() over String#match()".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::as_conversions)] // u32->usize is lossless on 32/64-bit
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        // Look for `something.match(arg)` used in a boolean context
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Check if callee is `something.match`
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property != "match" {
            return;
        }

        // Must have exactly one argument
        if call.arguments.len() != 1 {
            return;
        }

        // Check if the argument is a regex literal
        let Some(arg_id) = call.arguments.first() else {
            return;
        };
        let is_regex_arg = matches!(ctx.node(*arg_id), Some(AstNode::RegExpLiteral(_)));

        if is_regex_arg {
            let obj_id = member.object;
            let source = ctx.source_text();
            let obj_span = ctx.node(obj_id).map_or(
                starlint_ast::types::Span::EMPTY,
                starlint_ast::AstNode::span,
            );
            let str_text = source[obj_span.start as usize..obj_span.end as usize].to_owned();
            let regex_span = ctx.node(*arg_id).map_or(
                starlint_ast::types::Span::EMPTY,
                starlint_ast::AstNode::span,
            );
            let regex_text = source[regex_span.start as usize..regex_span.end as usize].to_owned();
            let replacement = format!("{regex_text}.test({str_text})");

            ctx.report(Diagnostic {
                rule_name: "prefer-regexp-test".to_owned(),
                message: "Prefer `RegExp#test()` over `String#match()`".to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some(format!("Replace with `{replacement}`")),
                fix: Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: format!("Replace with `{replacement}`"),
                    edits: vec![Edit {
                        span: Span::new(call.span.start, call.span.end),
                        replacement,
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferRegexpTest)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_match_with_regex() {
        let diags = lint("if (str.match(/foo/)) {}");
        assert_eq!(diags.len(), 1, "match with regex literal should be flagged");
    }

    #[test]
    fn test_allows_match_with_string() {
        let diags = lint("str.match('foo');");
        assert!(
            diags.is_empty(),
            "match with string argument should not be flagged"
        );
    }

    #[test]
    fn test_allows_test() {
        let diags = lint("/foo/.test(str);");
        assert!(diags.is_empty(), "test() should not be flagged");
    }

    #[test]
    fn test_allows_match_multiple_args() {
        let diags = lint("str.match(/foo/, 'g');");
        assert!(
            diags.is_empty(),
            "match with multiple args should not be flagged"
        );
    }
}
