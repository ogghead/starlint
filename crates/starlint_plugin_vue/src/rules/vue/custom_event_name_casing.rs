//! Rule: `vue/custom-event-name-casing`
//!
//! Enforce camelCase for custom event names in `$emit()` calls.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "vue/custom-event-name-casing";

/// Enforce camelCase for custom event names in `$emit()`.
#[derive(Debug)]
pub struct CustomEventNameCasing;

/// Convert a string to `camelCase`.
fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    for (i, ch) in s.chars().enumerate() {
        if ch == '-' || ch == '_' || ch == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(ch.to_uppercase());
            capitalize_next = false;
        } else if i == 0 {
            result.extend(ch.to_lowercase());
        } else {
            result.push(ch);
        }
    }
    result
}

/// Check if a string is `camelCase`.
fn is_camel_case(s: &str) -> bool {
    let first = s.chars().next();
    matches!(first, Some('a'..='z')) && !s.contains('-') && !s.contains('_') && !s.contains(' ')
}

impl LintRule for CustomEventNameCasing {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce camelCase for custom event names in `$emit()`".to_owned(),
            category: Category::Style,
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

        // Check for this.$emit() or $emit()
        let is_emit = match ctx.node(call.callee) {
            Some(AstNode::StaticMemberExpression(member)) => member.property.as_str() == "$emit",
            Some(AstNode::IdentifierReference(ident)) => ident.name.as_str() == "$emit",
            _ => false,
        };

        if !is_emit {
            return;
        }

        // Check first argument for casing — it should be a string literal
        let Some(first_arg_id) = call.arguments.first() else {
            return;
        };

        let Some(AstNode::StringLiteral(lit)) = ctx.node(*first_arg_id) else {
            return;
        };

        let event_name = lit.value.as_str();

        if !event_name.is_empty() && !is_camel_case(event_name) {
            // Fix: convert event name to camelCase
            let camel = to_camel_case(event_name);
            let fix = (camel != event_name).then(|| {
                // Replace just the string content (inside the quotes)
                let arg_span = lit.span;
                // The string literal span includes quotes, so skip them
                let inner_start = arg_span.start.saturating_add(1);
                let inner_end = arg_span.end.saturating_sub(1);
                Fix {
                    kind: FixKind::SafeFix,
                    message: format!("Rename to `{camel}`"),
                    edits: vec![Edit {
                        span: Span::new(inner_start, inner_end),
                        replacement: camel.clone(),
                    }],
                    is_snippet: false,
                }
            });

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!("Custom event name `{event_name}` should be camelCase"),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some(format!("Rename to `{camel}`")),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(CustomEventNameCasing)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_allows_camel_case_event() {
        let source = r#"this.$emit("myEvent", value);"#;
        let diags = lint(source);
        assert!(diags.is_empty(), "camelCase event name should be allowed");
    }

    #[test]
    fn test_flags_kebab_case_event() {
        let source = r#"this.$emit("my-event", value);"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "kebab-case event name should be flagged");
    }

    #[test]
    fn test_flags_pascal_case_event() {
        let source = r#"this.$emit("MyEvent", value);"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "PascalCase event name should be flagged");
    }
}
