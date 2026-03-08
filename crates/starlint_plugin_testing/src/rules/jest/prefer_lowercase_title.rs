//! Rule: `jest/prefer-lowercase-title`
//!
//! Suggest lowercase titles for `it`/`test` calls. Consistent lowercase
//! titles read more naturally as sentences: "it should work" vs
//! "it Should work".

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `it`/`test` calls with uppercase-starting titles.
#[derive(Debug)]
pub struct PreferLowercaseTitle;

impl LintRule for PreferLowercaseTitle {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-lowercase-title".to_owned(),
            description: "Suggest lowercase titles for `it`/`test` calls".to_owned(),
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

        // Must be `it(...)` or `test(...)` — not `describe`
        let callee_name = match ctx.node(call.callee) {
            Some(AstNode::IdentifierReference(id)) => id.name.as_str(),
            _ => return,
        };
        if callee_name != "it" && callee_name != "test" {
            return;
        }

        // First argument must be a string literal
        let Some(first_arg_id) = call.arguments.first() else {
            return;
        };
        let Some(AstNode::StringLiteral(title)) = ctx.node(*first_arg_id) else {
            return;
        };
        let title_str = title.value.as_str();

        // Check if the first character is uppercase
        let Some(first_char) = title_str.chars().next() else {
            return;
        };
        if first_char.is_uppercase() {
            // Replace just the first character inside the string literal
            // title.span includes quotes, so the first content char is at start+1
            let char_start = title.span.start.saturating_add(1);
            let char_end =
                char_start.saturating_add(u32::try_from(first_char.len_utf8()).unwrap_or(1));
            let lowered: String = first_char.to_lowercase().collect();
            ctx.report(Diagnostic {
                rule_name: "jest/prefer-lowercase-title".to_owned(),
                message: "Test titles should start with a lowercase letter".to_owned(),
                span: Span::new(title.span.start, title.span.end),
                severity: Severity::Warning,
                help: Some("Lowercase the first letter of the test title".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Lowercase first letter".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(char_start, char_end),
                        replacement: lowered,
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
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferLowercaseTitle)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_uppercase_title() {
        let diags = lint(r"test('Should work', () => {});");
        assert_eq!(
            diags.len(),
            1,
            "uppercase-starting test title should be flagged"
        );
    }

    #[test]
    fn test_allows_lowercase_title() {
        let diags = lint(r"test('should work', () => {});");
        assert!(
            diags.is_empty(),
            "lowercase-starting test title should not be flagged"
        );
    }

    #[test]
    fn test_allows_describe_uppercase() {
        let diags = lint(r"describe('MyComponent', () => {});");
        assert!(
            diags.is_empty(),
            "`describe` with uppercase title should not be flagged"
        );
    }
}
