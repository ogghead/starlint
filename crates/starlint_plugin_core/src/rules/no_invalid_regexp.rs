//! Rule: `no-invalid-regexp`
//!
//! Disallow invalid regular expression strings in `RegExp` constructors.
//! An invalid regex will throw at runtime and is almost always a bug.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `new RegExp(...)` calls with invalid regex patterns.
#[derive(Debug)]
pub struct NoInvalidRegexp;

impl LintRule for NoInvalidRegexp {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-invalid-regexp".to_owned(),
            description: "Disallow invalid regular expression strings in RegExp constructors"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::NewExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::NewExpression(new_expr) = node else {
            return;
        };

        // Check if it's `new RegExp(...)`
        let Some(AstNode::IdentifierReference(ident)) = ctx.node(new_expr.callee) else {
            return;
        };

        if ident.name != "RegExp" {
            return;
        }

        // Get the first argument (the pattern)
        let Some(first_arg_id) = new_expr.arguments.first() else {
            return;
        };

        let Some(AstNode::StringLiteral(pattern_lit)) = ctx.node(*first_arg_id) else {
            return;
        };

        let pattern = pattern_lit.value.as_str();

        // Get flags if present
        let flags = new_expr
            .arguments
            .get(1)
            .and_then(|arg_id| {
                if let Some(AstNode::StringLiteral(s)) = ctx.node(*arg_id) {
                    Some(s.value.as_str())
                } else {
                    None
                }
            })
            .unwrap_or("");

        // Validate the regex pattern
        if let Some(error) = validate_regex_pattern(pattern, flags) {
            ctx.report(Diagnostic {
                rule_name: "no-invalid-regexp".to_owned(),
                message: format!("Invalid regular expression: {error}"),
                span: Span::new(new_expr.span.start, new_expr.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Simple validation of regex patterns. Returns an error message if invalid.
fn validate_regex_pattern(pattern: &str, flags: &str) -> Option<String> {
    // Check for invalid flags
    for ch in flags.chars() {
        if !matches!(ch, 'd' | 'g' | 'i' | 'm' | 's' | 'u' | 'v' | 'y') {
            return Some(format!(
                "Invalid flags supplied to RegExp constructor '{ch}'"
            ));
        }
    }

    // Check for duplicate flags
    let mut seen_flags = [false; 128];
    for ch in flags.bytes() {
        let idx = usize::from(ch);
        if idx < 128 {
            if *seen_flags.get(idx).unwrap_or(&false) {
                return Some(format!(
                    "Duplicate flag '{}' in RegExp constructor",
                    char::from(ch)
                ));
            }
            if let Some(slot) = seen_flags.get_mut(idx) {
                *slot = true;
            }
        }
    }

    // Check for unbalanced parentheses
    let mut paren_depth: i32 = 0;
    let mut bracket_depth: i32 = 0;
    let mut prev_was_escape = false;

    for ch in pattern.chars() {
        if prev_was_escape {
            prev_was_escape = false;
            continue;
        }
        if ch == '\\' {
            prev_was_escape = true;
            continue;
        }
        if bracket_depth > 0 {
            if ch == ']' {
                bracket_depth = bracket_depth.saturating_sub(1);
            }
            continue;
        }
        match ch {
            '(' => paren_depth = paren_depth.saturating_add(1),
            ')' => {
                paren_depth = paren_depth.saturating_sub(1);
                if paren_depth < 0 {
                    return Some("Unmatched ')'".to_owned());
                }
            }
            '[' => bracket_depth = bracket_depth.saturating_add(1),
            _ => {}
        }
    }

    if paren_depth != 0 {
        return Some("Unterminated group".to_owned());
    }

    None
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoInvalidRegexp)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_allows_valid_regexp() {
        let diags = lint("new RegExp('abc');");
        assert!(diags.is_empty(), "valid regex should not be flagged");
    }

    #[test]
    fn test_flags_invalid_flags() {
        let diags = lint("new RegExp('abc', 'z');");
        assert_eq!(diags.len(), 1, "invalid flags should be flagged");
    }

    #[test]
    fn test_flags_duplicate_flags() {
        let diags = lint("new RegExp('abc', 'gg');");
        assert_eq!(diags.len(), 1, "duplicate flags should be flagged");
    }

    #[test]
    fn test_flags_unbalanced_parens() {
        let diags = lint("new RegExp('(abc');");
        assert_eq!(diags.len(), 1, "unbalanced parens should be flagged");
    }

    #[test]
    fn test_allows_valid_flags() {
        let diags = lint("new RegExp('abc', 'gi');");
        assert!(diags.is_empty(), "valid flags should not be flagged");
    }
}
