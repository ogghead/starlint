//! Rule: `new-cap`
//!
//! Require constructor names to begin with a capital letter. Calling `new` on
//! a lowercase identifier is almost always a mistake — constructors should
//! follow the `PascalCase` convention.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `new` expressions where the callee starts with a lowercase letter.
#[derive(Debug)]
pub struct NewCap;

impl LintRule for NewCap {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "new-cap".to_owned(),
            description: "Require constructor names to begin with a capital letter".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::NewExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::NewExpression(new_expr) = node else {
            return;
        };

        // Determine the relevant name to check:
        // - `new foo()` -> check "foo"
        // - `new foo.Bar()` -> check "Bar" (the last property)
        // - `new foo.bar()` -> check "bar"
        let (callee_name, callee_span) = match ctx.node(new_expr.callee) {
            Some(AstNode::IdentifierReference(ident)) => {
                (ident.name.as_str().to_owned(), Some(ident.span))
            }
            Some(AstNode::StaticMemberExpression(member)) => (member.property.clone(), None),
            _ => return,
        };

        // Check if the first character is lowercase
        let first_char = callee_name.chars().next();
        let Some(ch) = first_char else {
            return;
        };

        if ch.is_lowercase() {
            // Fix: capitalize the first letter of the constructor name
            let fix = if let Some(span) = callee_span {
                let capitalized: String = ch
                    .to_uppercase()
                    .chain(callee_name.chars().skip(1))
                    .collect();
                Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: format!("Capitalize to `{capitalized}`"),
                    edits: vec![Edit {
                        span: Span::new(span.start, span.end),
                        replacement: capitalized,
                    }],
                    is_snippet: false,
                })
            } else {
                // StaticMemberExpression property is a String, no span available for just the property
                None
            };

            ctx.report(Diagnostic {
                rule_name: "new-cap".to_owned(),
                message: format!(
                    "A constructor name `{callee_name}` should start with an uppercase letter"
                ),
                span: Span::new(new_expr.span.start, new_expr.span.end),
                severity: Severity::Warning,
                help: None,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NewCap)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_lowercase_constructor() {
        let diags = lint("var x = new foo();");
        assert_eq!(diags.len(), 1, "new foo() with lowercase should be flagged");
    }

    #[test]
    fn test_allows_uppercase_constructor() {
        let diags = lint("var x = new Foo();");
        assert!(diags.is_empty(), "new Foo() should not be flagged");
    }

    #[test]
    fn test_allows_member_expression_uppercase() {
        let diags = lint("var x = new bar.Baz();");
        assert!(
            diags.is_empty(),
            "new bar.Baz() should not be flagged (checks last property)"
        );
    }

    #[test]
    fn test_flags_member_expression_lowercase() {
        let diags = lint("var x = new bar.baz();");
        assert_eq!(
            diags.len(),
            1,
            "new bar.baz() with lowercase property should be flagged"
        );
    }

    #[test]
    fn test_allows_date_constructor() {
        let diags = lint("var d = new Date();");
        assert!(diags.is_empty(), "new Date() should not be flagged");
    }

    #[test]
    fn test_allows_regular_function_call() {
        let diags = lint("foo();");
        assert!(
            diags.is_empty(),
            "regular function call should not be flagged"
        );
    }

    #[test]
    fn test_allows_uppercase_function_call() {
        let diags = lint("Foo();");
        assert!(
            diags.is_empty(),
            "uppercase function call without new should not be flagged"
        );
    }
}
