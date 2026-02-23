//! Rule: `better-regex` (unicorn)
//!
//! Flags regular expressions that contain character classes which can be
//! replaced with shorter built-in shorthand classes (e.g. `[0-9]` to `\d`).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Known improvable character class patterns and their replacements.
const IMPROVABLE_PATTERNS: &[(&str, &str)] = &[
    ("[0-9]", "\\d"),
    ("[^0-9]", "\\D"),
    ("[a-zA-Z0-9_]", "\\w"),
    ("[A-Za-z0-9_]", "\\w"),
    ("[^a-zA-Z0-9_]", "\\W"),
    ("[^A-Za-z0-9_]", "\\W"),
    (
        "[a-zA-Z]",
        "[a-zA-Z] (consider \\w or a unicode-aware class)",
    ),
    (
        "[A-Za-z]",
        "[A-Za-z] (consider \\w or a unicode-aware class)",
    ),
];

/// Patterns that can be directly replaced (the replacement is a valid regex shorthand).
const FIXABLE_PATTERNS: &[(&str, &str)] = &[
    ("[0-9]", "\\d"),
    ("[^0-9]", "\\D"),
    ("[a-zA-Z0-9_]", "\\w"),
    ("[A-Za-z0-9_]", "\\w"),
    ("[^a-zA-Z0-9_]", "\\W"),
    ("[^A-Za-z0-9_]", "\\W"),
];

/// Flags regex literals with character classes that have shorter alternatives.
#[derive(Debug)]
pub struct BetterRegex;

impl LintRule for BetterRegex {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "better-regex".to_owned(),
            description: "Suggest simpler alternatives for regex character classes".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::RegExpLiteral])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::RegExpLiteral(regex) = node else {
            return;
        };

        let pattern = regex.pattern.as_str();

        for &(class, replacement) in IMPROVABLE_PATTERNS {
            if pattern.contains(class) {
                // Build fix for patterns with direct shorthand replacements
                #[allow(clippy::as_conversions)]
                let fix = FIXABLE_PATTERNS.iter().find(|(c, _)| *c == class).and_then(
                    |(_, shorthand)| {
                        let source = ctx.source_text();
                        let regex_text =
                            source.get(regex.span.start as usize..regex.span.end as usize)?;
                        let new_regex = regex_text.replacen(class, shorthand, 1);
                        Some(Fix {
                            kind: FixKind::SafeFix,
                            message: format!("Replace `{class}` with `{shorthand}`"),
                            edits: vec![Edit {
                                span: Span::new(regex.span.start, regex.span.end),
                                replacement: new_regex,
                            }],
                            is_snippet: false,
                        })
                    },
                );

                ctx.report(Diagnostic {
                    rule_name: "better-regex".to_owned(),
                    message: format!(
                        "The regex pattern `{class}` can be replaced with `{replacement}`"
                    ),
                    span: Span::new(regex.span.start, regex.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix,
                    labels: vec![],
                });
                // Report only the first match per regex to avoid noise.
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(BetterRegex)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_digit_class() {
        let diags = lint("var re = /[0-9]+/;");
        assert_eq!(diags.len(), 1, "[0-9] should be flagged (suggest \\d)");
    }

    #[test]
    fn test_flags_word_class() {
        let diags = lint("var re = /[a-zA-Z0-9_]/;");
        assert_eq!(
            diags.len(),
            1,
            "[a-zA-Z0-9_] should be flagged (suggest \\w)"
        );
    }

    #[test]
    fn test_flags_negated_digit_class() {
        let diags = lint("var re = /[^0-9]/;");
        assert_eq!(diags.len(), 1, "[^0-9] should be flagged (suggest \\D)");
    }

    #[test]
    fn test_flags_negated_word_class() {
        let diags = lint("var re = /[^a-zA-Z0-9_]/;");
        assert_eq!(
            diags.len(),
            1,
            "[^a-zA-Z0-9_] should be flagged (suggest \\W)"
        );
    }

    #[test]
    fn test_flags_alpha_class() {
        let diags = lint("var re = /[a-zA-Z]/;");
        assert_eq!(diags.len(), 1, "[a-zA-Z] should be flagged");
    }

    #[test]
    fn test_allows_shorthand_digit() {
        let diags = lint("var re = /\\d+/;");
        assert!(diags.is_empty(), "\\d should not be flagged");
    }

    #[test]
    fn test_allows_partial_range() {
        let diags = lint("var re = /[a-z]/;");
        assert!(diags.is_empty(), "[a-z] alone should not be flagged");
    }

    #[test]
    fn test_allows_custom_range() {
        let diags = lint("var re = /[0-8]/;");
        assert!(
            diags.is_empty(),
            "[0-8] should not be flagged (not the same as [0-9])"
        );
    }

    #[test]
    fn test_allows_normal_regex() {
        let diags = lint("var re = /foo|bar/;");
        assert!(diags.is_empty(), "normal regex should not be flagged");
    }

    #[test]
    fn test_reports_only_first_match() {
        // This regex has both [0-9] and [a-zA-Z0-9_] — only report one.
        let diags = lint("var re = /[0-9][a-zA-Z0-9_]/;");
        assert_eq!(
            diags.len(),
            1,
            "should report only the first improvable pattern per regex"
        );
    }
}
