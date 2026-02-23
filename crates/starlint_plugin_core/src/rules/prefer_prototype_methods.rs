//! Rule: `prefer-prototype-methods`
//!
//! Prefer using prototype methods directly instead of creating temporary
//! instances. Patterns like `[].forEach.call(obj, fn)` or
//! `"".trim.call(str)` create a throwaway literal just to access a
//! prototype method. Use `Array.prototype.forEach.call(obj, fn)` instead.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.call()`/`.apply()` on methods accessed from empty array or string
/// literals.
#[derive(Debug)]
pub struct PreferPrototypeMethods;

impl LintRule for PreferPrototypeMethods {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-prototype-methods".to_owned(),
            description: "Prefer prototype methods over creating instances to access them"
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

        // Callee must be `<something>.call(...)` or `<something>.apply(...)`
        let Some(AstNode::StaticMemberExpression(outer_member)) = ctx.node(call.callee) else {
            return;
        };

        let method = outer_member.property.as_str();
        if method != "call" && method != "apply" {
            return;
        }

        // The object of the outer `.call()`/`.apply()` must itself be a
        // member expression: `[].forEach` or `"".trim`
        let Some(AstNode::StaticMemberExpression(inner_member)) = ctx.node(outer_member.object)
        else {
            return;
        };

        // The innermost object must be an empty array literal or empty string
        // literal.
        let inner_obj = ctx.node(inner_member.object);
        let is_empty_array = matches!(
            inner_obj,
            Some(AstNode::ArrayExpression(arr)) if arr.elements.is_empty()
        );

        let is_empty_string = matches!(
            inner_obj,
            Some(AstNode::StringLiteral(s)) if s.value.is_empty()
        );

        if !is_empty_array && !is_empty_string {
            return;
        }

        let prototype_method = inner_member.property.as_str();
        let prototype_owner = if is_empty_array { "Array" } else { "String" };

        // Replace the literal (`[]` or `""`) with `Type.prototype`
        let literal_span = inner_obj.map_or(
            starlint_ast::types::Span::EMPTY,
            starlint_ast::AstNode::span,
        );
        let replacement = format!("{prototype_owner}.prototype");

        ctx.report(Diagnostic {
            rule_name: "prefer-prototype-methods".to_owned(),
            message: format!(
                "Use `{prototype_owner}.prototype.{prototype_method}.{method}()` instead of a \
                 literal"
            ),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some(format!(
                "Replace with `{prototype_owner}.prototype.{prototype_method}.{method}()`"
            )),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: format!(
                    "Replace with `{prototype_owner}.prototype.{prototype_method}.{method}()`"
                ),
                edits: vec![Edit {
                    span: Span::new(literal_span.start, literal_span.end),
                    replacement,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferPrototypeMethods)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_array_foreach_call() {
        let diags = lint("[].forEach.call(obj, fn);");
        assert_eq!(diags.len(), 1, "[].forEach.call() should be flagged");
    }

    #[test]
    fn test_flags_array_map_call() {
        let diags = lint("[].map.call(obj, fn);");
        assert_eq!(diags.len(), 1, "[].map.call() should be flagged");
    }

    #[test]
    fn test_flags_string_trim_call() {
        let diags = lint(r#""".trim.call(str);"#);
        assert_eq!(diags.len(), 1, r#""".trim.call() should be flagged"#);
    }

    #[test]
    fn test_allows_prototype_call() {
        let diags = lint("Array.prototype.forEach.call(obj, fn);");
        assert!(
            diags.is_empty(),
            "Array.prototype.forEach.call() should not be flagged"
        );
    }

    #[test]
    fn test_allows_normal_method_call() {
        let diags = lint("arr.forEach(fn);");
        assert!(diags.is_empty(), "normal method call should not be flagged");
    }

    #[test]
    fn test_allows_non_empty_array() {
        let diags = lint("[1].forEach.call(obj, fn);");
        assert!(
            diags.is_empty(),
            "non-empty array literal should not be flagged"
        );
    }

    #[test]
    fn test_flags_array_apply() {
        let diags = lint("[].slice.apply(obj, [1, 2]);");
        assert_eq!(diags.len(), 1, "[].slice.apply() should be flagged");
    }
}
