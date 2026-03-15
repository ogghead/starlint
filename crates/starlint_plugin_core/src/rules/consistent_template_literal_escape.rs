//! Rule: `consistent-template-literal-escape` (unicorn)
//!
//! Flags unnecessary escape sequences in template literals. Single quotes
//! (`\'`) and double quotes (`\"`) do not need escaping inside template
//! literals (backtick-delimited strings), so their escaped forms are
//! unnecessary noise.

#![allow(dead_code)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags unnecessary `\'` or `\"` escapes in template literals.
#[derive(Debug)]
pub struct ConsistentTemplateLiteralEscape;

/// Check if a raw template quasi string contains unnecessary escape sequences.
///
/// Looks for `\'` or `\"` which do not need escaping in template literals.
fn has_unnecessary_escape(raw: &str) -> bool {
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.peek() {
                Some('\'' | '"') => return true,
                Some(_) => {
                    // Skip the escaped character
                    let _skip = chars.next();
                }
                None => {}
            }
        }
    }
    false
}

/// Remove unnecessary `\'` and `\"` escape sequences from raw template text.
fn remove_unnecessary_escapes(raw: &str) -> String {
    let mut result = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' && matches!(chars.peek(), Some('\'' | '"')) {
            // Skip the backslash, the quote will be added by the next iteration
        } else {
            result.push(ch);
        }
    }
    result
}

// Note: In starlint_ast, quasis are Box<[String]> (raw strings directly),
// not template elements with spans. We cannot compute per-quasi spans without
// source positions, so fixes operate on the whole template span.

impl LintRule for ConsistentTemplateLiteralEscape {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "consistent-template-literal-escape".to_owned(),
            description:
                "Disallow unnecessary escape sequences `\\'` and `\\\"` in template literals"
                    .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TemplateLiteral])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::TemplateLiteral(template) = node else {
            return;
        };

        let mut found = false;
        for quasi in &*template.quasis {
            let raw = quasi.as_str();
            if has_unnecessary_escape(raw) {
                found = true;
            }
        }

        if found {
            let fix: Option<Fix> = None;

            ctx.report(Diagnostic {
                rule_name: "consistent-template-literal-escape".to_owned(),
                message:
                    "Unnecessary escape sequence in template literal — `\\'` and `\\\"` do not \
                     need escaping in template literals"
                        .to_owned(),
                span: Span::new(template.span.start, template.span.end),
                severity: Severity::Warning,
                help: Some(
                    "Remove the backslash — single and double quotes don't need escaping in \
                     template literals"
                        .to_owned(),
                ),
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(ConsistentTemplateLiteralEscape);

    #[test]
    fn test_flags_escaped_single_quote() {
        let diags = lint(r"var x = `hello \'world\'`;");
        assert_eq!(
            diags.len(),
            1,
            "escaped single quotes in template literal should be flagged"
        );
    }

    #[test]
    fn test_flags_escaped_double_quote() {
        let diags = lint(r#"var x = `say \"hi\"`;"#);
        assert_eq!(
            diags.len(),
            1,
            "escaped double quotes in template literal should be flagged"
        );
    }

    #[test]
    fn test_allows_escaped_backtick() {
        let diags = lint(r"var x = `hello \`world\``;");
        assert!(
            diags.is_empty(),
            "escaped backticks in template literal should not be flagged (they are necessary)"
        );
    }

    #[test]
    fn test_allows_template_with_expression() {
        let diags = lint("var x = `hello ${name}`;");
        assert!(
            diags.is_empty(),
            "plain template literal with expression should not be flagged"
        );
    }

    #[test]
    fn test_allows_plain_template() {
        let diags = lint("var x = `hello world`;");
        assert!(
            diags.is_empty(),
            "plain template literal should not be flagged"
        );
    }

    #[test]
    fn test_allows_other_escapes() {
        let diags = lint(r"var x = `hello\nworld`;");
        assert!(
            diags.is_empty(),
            "newline escape in template literal should not be flagged"
        );
    }

    #[test]
    fn test_allows_tab_escape() {
        let diags = lint(r"var x = `col1\tcol2`;");
        assert!(
            diags.is_empty(),
            "tab escape in template literal should not be flagged"
        );
    }

    #[test]
    fn test_flags_only_once_per_template() {
        let diags = lint(r"var x = `\'hello\' and \'world\'`;");
        assert_eq!(
            diags.len(),
            1,
            "should report only once per template literal even with multiple occurrences"
        );
    }
}
