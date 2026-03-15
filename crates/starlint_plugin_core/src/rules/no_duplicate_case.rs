//! Rule: `no-duplicate-case`
//!
//! Disallow duplicate case labels in `switch` statements. If a `switch`
//! statement has duplicate case expressions, the second case will never
//! be reached (the first matching case always wins).

use std::collections::HashSet;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `switch` statements with duplicate `case` labels.
#[derive(Debug)]
pub struct NoDuplicateCase;

impl LintRule for NoDuplicateCase {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-duplicate-case".to_owned(),
            description: "Disallow duplicate case labels".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::SwitchStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::SwitchStatement(switch) = node else {
            return;
        };

        // Collect case info: (case_span, test_span, test_source_text)
        // We resolve NodeIds upfront to avoid borrow conflicts later
        let cases_info: Vec<(Span, Option<Span>)> = switch
            .cases
            .iter()
            .filter_map(|&case_id| {
                let case = ctx.node(case_id)?.as_switch_case()?;
                let case_span = Span::new(case.span.start, case.span.end);
                let test_span = case.test.and_then(|test_id| {
                    let ts = ctx.node(test_id)?.span();
                    Some(Span::new(ts.start, ts.end))
                });
                Some((case_span, test_span))
            })
            .collect();

        let mut seen = HashSet::new();

        for &(case_span, maybe_test_span) in &cases_info {
            let Some(test_span) = maybe_test_span else {
                // `default:` has no test expression
                continue;
            };

            // Use the source text of the test expression as the key for
            // duplicate detection.
            let start = usize::try_from(test_span.start).unwrap_or(0);
            let end = usize::try_from(test_span.end).unwrap_or(0);
            let Some(source_slice) = ctx.source_text().get(start..end) else {
                continue;
            };
            let key = source_slice.to_owned();

            if !seen.insert(key.clone()) {
                // Fix: delete the entire duplicate case clause
                let fix = Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Remove duplicate case clause".to_owned(),
                    edits: vec![Edit {
                        span: case_span,
                        replacement: String::new(),
                    }],
                    is_snippet: false,
                });
                ctx.report(Diagnostic {
                    rule_name: "no-duplicate-case".to_owned(),
                    message: format!("Duplicate case label `{key}`"),
                    span: test_span,
                    severity: Severity::Error,
                    help: None,
                    fix,
                    labels: vec![],
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    starlint_rule_framework::lint_rule_test!(NoDuplicateCase);

    #[test]
    fn test_flags_duplicate_case() {
        let diags = lint("switch(x) { case 1: break; case 1: break; }");
        assert_eq!(diags.len(), 1, "duplicate case 1 should be flagged");
    }

    #[test]
    fn test_flags_duplicate_string_case() {
        let diags = lint(r#"switch(x) { case "a": break; case "a": break; }"#);
        assert_eq!(diags.len(), 1, "duplicate string case should be flagged");
    }

    #[test]
    fn test_flags_multiple_duplicates() {
        let diags =
            lint("switch(x) { case 1: break; case 2: break; case 1: break; case 2: break; }");
        assert_eq!(
            diags.len(),
            2,
            "two pairs of duplicates should produce two diagnostics"
        );
    }

    #[test]
    fn test_allows_unique_cases() {
        let diags = lint("switch(x) { case 1: break; case 2: break; case 3: break; }");
        assert!(diags.is_empty(), "unique cases should not be flagged");
    }

    #[test]
    fn test_allows_default_case() {
        let diags = lint("switch(x) { case 1: break; default: break; }");
        assert!(diags.is_empty(), "default case should not be flagged");
    }

    #[test]
    fn test_allows_duplicate_identifier_different_names() {
        let diags = lint("switch(x) { case a: break; case b: break; }");
        assert!(
            diags.is_empty(),
            "different identifiers should not be flagged"
        );
    }
}
