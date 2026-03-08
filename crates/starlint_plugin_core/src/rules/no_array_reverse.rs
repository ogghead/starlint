//! Rule: `no-array-reverse`
//!
//! Flag `.reverse()` which mutates the array in-place. Prefer the
//! non-mutating `.toReversed()` method instead.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.reverse()` calls which mutate the original array.
#[derive(Debug)]
pub struct NoArrayReverse;

impl LintRule for NoArrayReverse {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-array-reverse".to_owned(),
            description: "Disallow `.reverse()` which mutates the array — prefer `.toReversed()`"
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

        if member.property.as_str() != "reverse" {
            return;
        }

        if !call.arguments.is_empty() {
            return;
        }

        // Fix: replace `.reverse` with `.toReversed` in the property name
        // property is a String, compute span from member.span.end (before `(`) minus property length
        let prop_len = u32::try_from(member.property.len()).unwrap_or(0);
        let prop_end = member.span.end;
        let prop_start = prop_end.saturating_sub(prop_len);
        let fix = Some(Fix {
            kind: FixKind::SuggestionFix,
            message: "Replace `.reverse()` with `.toReversed()`".to_owned(),
            edits: vec![Edit {
                span: Span::new(prop_start, prop_end),
                replacement: "toReversed".to_owned(),
            }],
            is_snippet: false,
        });

        ctx.report(Diagnostic {
            rule_name: "no-array-reverse".to_owned(),
            message: "`.reverse()` mutates the array — consider `.toReversed()` instead".to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some("Replace `.reverse()` with `.toReversed()`".to_owned()),
            fix,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoArrayReverse)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_reverse_no_args() {
        let diags = lint("arr.reverse();");
        assert_eq!(diags.len(), 1, ".reverse() should be flagged");
    }

    #[test]
    fn test_allows_to_reversed() {
        let diags = lint("arr.toReversed();");
        assert!(diags.is_empty(), ".toReversed() should not be flagged");
    }

    #[test]
    fn test_flags_str_reverse() {
        // Without type information we cannot distinguish str.reverse() from arr.reverse()
        let diags = lint("str.reverse();");
        assert_eq!(
            diags.len(),
            1,
            "str.reverse() should be flagged (no type info)"
        );
    }

    #[test]
    fn test_allows_reverse_with_args() {
        let diags = lint("arr.reverse(true);");
        assert!(
            diags.is_empty(),
            ".reverse() with arguments should not be flagged"
        );
    }
}
