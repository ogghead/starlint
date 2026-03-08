//! Rule: `prefer-string-replace-all` (unicorn)
//!
//! Prefer `String#replaceAll()` over `String#replace()` with a global
//! regex. Using `replaceAll` is more readable and clearly communicates
//! the intent to replace all occurrences.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `str.replace(/regex/g, ...)` that could use `str.replaceAll(...)`.
#[derive(Debug)]
pub struct PreferStringReplaceAll;

impl LintRule for PreferStringReplaceAll {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-string-replace-all".to_owned(),
            description: "Prefer String#replaceAll() over String#replace() with global regex"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Check for `something.replace(regex, replacement)`
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property != "replace" {
            return;
        }

        // Must have at least 2 arguments
        if call.arguments.len() < 2 {
            return;
        }

        // First argument must be a regex with global flag
        let Some(first_arg_id) = call.arguments.first() else {
            return;
        };

        let Some(AstNode::RegExpLiteral(regex)) = ctx.node(*first_arg_id) else {
            return;
        };

        // Check if the regex has the global flag
        if regex.flags.contains('g') {
            let call_span = Span::new(call.span.start, call.span.end);
            // Fix: rename `replace` to `replaceAll` in the method name.
            // Since property is a String (no span), use source text to locate it.
            let source = ctx.source_text();
            let call_text = source
                .get(call.span.start as usize..call.span.end as usize)
                .unwrap_or("");
            // Find ".replace(" in the call text to get the property span
            let fix = call_text.find(".replace(").map(|offset| {
                let prop_start = call
                    .span
                    .start
                    .saturating_add(offset as u32)
                    .saturating_add(1);
                let prop_end = prop_start.saturating_add(7); // "replace" is 7 chars
                let prop_span = Span::new(prop_start, prop_end);
                Fix {
                    kind: FixKind::SuggestionFix,
                    message: "Replace `replace` with `replaceAll`".to_owned(),
                    edits: vec![Edit {
                        span: prop_span,
                        replacement: "replaceAll".to_owned(),
                    }],
                    is_snippet: false,
                }
            });
            ctx.report(Diagnostic {
                rule_name: "prefer-string-replace-all".to_owned(),
                message: "Prefer `String#replaceAll()` over `String#replace()` with a global regex"
                    .to_owned(),
                span: call_span,
                severity: Severity::Warning,
                help: Some("Use `replaceAll` with a string pattern instead".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferStringReplaceAll)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_replace_with_global_regex() {
        let diags = lint("str.replace(/foo/g, 'bar');");
        assert_eq!(
            diags.len(),
            1,
            "replace with global regex should be flagged"
        );
    }

    #[test]
    fn test_allows_replace_without_global() {
        let diags = lint("str.replace(/foo/, 'bar');");
        assert!(
            diags.is_empty(),
            "replace without global flag should not be flagged"
        );
    }

    #[test]
    fn test_allows_replace_with_string() {
        let diags = lint("str.replace('foo', 'bar');");
        assert!(
            diags.is_empty(),
            "replace with string should not be flagged"
        );
    }

    #[test]
    fn test_allows_replace_all() {
        let diags = lint("str.replaceAll('foo', 'bar');");
        assert!(diags.is_empty(), "replaceAll should not be flagged");
    }
}
