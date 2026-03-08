//! Rule: `consistent-assert`
//!
//! Prefer strict assertion methods over loose ones.
//! Use `assert.strictEqual` instead of `assert.equal`,
//! `assert.notStrictEqual` instead of `assert.notEqual`,
//! `assert.deepStrictEqual` instead of `assert.deepEqual`,
//! and `assert.notDeepStrictEqual` instead of `assert.notDeepEqual`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags loose assertion methods on the `assert` object.
#[derive(Debug)]
pub struct ConsistentAssert;

/// Map from loose method name to its strict equivalent.
fn strict_equivalent(method: &str) -> Option<&'static str> {
    match method {
        "equal" => Some("strictEqual"),
        "notEqual" => Some("notStrictEqual"),
        "deepEqual" => Some("deepStrictEqual"),
        "notDeepEqual" => Some("notDeepStrictEqual"),
        _ => None,
    }
}

impl LintRule for ConsistentAssert {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "consistent-assert".to_owned(),
            description: "Prefer strict assertion methods".to_owned(),
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

        // Check that the object is `assert`
        let is_assert = matches!(
            ctx.node(member.object),
            Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "assert"
        );

        if !is_assert {
            return;
        }

        let method = member.property.as_str();
        let Some(replacement) = strict_equivalent(method) else {
            return;
        };

        // Compute the span of the property name from source text
        // The property starts after the '.' in the member expression
        let source = ctx.source_text();
        let member_start = usize::try_from(member.span.start).unwrap_or(0);
        let member_end = usize::try_from(member.span.end).unwrap_or(0);
        let member_text = source.get(member_start..member_end).unwrap_or("");
        let dot_offset = member_text.rfind('.').unwrap_or(0);
        let prop_start = member
            .span
            .start
            .saturating_add(u32::try_from(dot_offset).unwrap_or(0))
            .saturating_add(1);
        let prop_end = member.span.end;

        ctx.report(Diagnostic {
            rule_name: "consistent-assert".to_owned(),
            message: format!("Use `assert.{replacement}()` instead of `assert.{method}()`"),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some(format!(
                "Replace `assert.{method}` with `assert.{replacement}` for strict comparison"
            )),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: format!("Replace `{method}` with `{replacement}`"),
                edits: vec![Edit {
                    span: Span::new(prop_start, prop_end),
                    replacement: replacement.to_owned(),
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
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ConsistentAssert)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_equal() {
        let diags = lint("assert.equal(a, b);");
        assert_eq!(diags.len(), 1, "assert.equal should be flagged");
    }

    #[test]
    fn test_flags_not_equal() {
        let diags = lint("assert.notEqual(a, b);");
        assert_eq!(diags.len(), 1, "assert.notEqual should be flagged");
    }

    #[test]
    fn test_flags_deep_equal() {
        let diags = lint("assert.deepEqual(a, b);");
        assert_eq!(diags.len(), 1, "assert.deepEqual should be flagged");
    }

    #[test]
    fn test_flags_not_deep_equal() {
        let diags = lint("assert.notDeepEqual(a, b);");
        assert_eq!(diags.len(), 1, "assert.notDeepEqual should be flagged");
    }

    #[test]
    fn test_allows_strict_equal() {
        let diags = lint("assert.strictEqual(a, b);");
        assert!(diags.is_empty(), "assert.strictEqual should not be flagged");
    }

    #[test]
    fn test_allows_deep_strict_equal() {
        let diags = lint("assert.deepStrictEqual(a, b);");
        assert!(
            diags.is_empty(),
            "assert.deepStrictEqual should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_assert_object() {
        let diags = lint("foo.equal(a, b);");
        assert!(diags.is_empty(), "non-assert object should not be flagged");
    }

    #[test]
    fn test_allows_other_assert_methods() {
        let diags = lint("assert.ok(a);");
        assert!(diags.is_empty(), "assert.ok should not be flagged");
    }
}
