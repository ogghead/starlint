//! Rule: `default-case-last`
//!
//! Require the `default` case in switch statements to be the last case.
//! Placing the default case in the middle of a switch makes it harder to
//! understand the flow, because you have to mentally skip over it when
//! reading the subsequent cases.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `default` cases that are not the last case in a switch statement.
#[derive(Debug)]
pub struct DefaultCaseLast;

impl LintRule for DefaultCaseLast {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "default-case-last".to_owned(),
            description: "Require `default` case to be last in `switch` statements".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::SwitchStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::SwitchStatement(switch) = node else {
            return;
        };

        let case_count = switch.cases.len();

        // No cases or only one case — nothing to flag
        if case_count <= 1 {
            return;
        }

        // Collect case info upfront to avoid borrow conflicts
        let cases_info: Vec<(Span, bool)> = switch
            .cases
            .iter()
            .filter_map(|&case_id| {
                let case = ctx.node(case_id)?.as_switch_case()?;
                Some((
                    Span::new(case.span.start, case.span.end),
                    case.test.is_none(),
                ))
            })
            .collect();

        let info_count = cases_info.len();

        for (i, &(case_span, is_default)) in cases_info.iter().enumerate() {
            let is_last = i.saturating_add(1) >= info_count;

            if is_default && !is_last {
                // Fix: move the default case to the end by deleting it from
                // its current position and inserting it after the last case.
                #[allow(clippy::as_conversions)]
                let fix = cases_info.last().and_then(|&(last_span, _)| {
                    let source = ctx.source_text();
                    let default_text =
                        source.get(case_span.start as usize..case_span.end as usize)?;
                    Some(Fix {
                        kind: FixKind::SuggestionFix,
                        message: "Move `default` case to the end".to_owned(),
                        edits: vec![
                            // Delete the default case from current position
                            Edit {
                                span: case_span,
                                replacement: String::new(),
                            },
                            // Insert it after the last case
                            Edit {
                                span: Span::new(last_span.end, last_span.end),
                                replacement: format!(" {default_text}"),
                            },
                        ],
                        is_snippet: false,
                    })
                });

                ctx.report(Diagnostic {
                    rule_name: "default-case-last".to_owned(),
                    message: "The `default` case should be the last case in a `switch` statement"
                        .to_owned(),
                    span: case_span,
                    severity: Severity::Warning,
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

    starlint_rule_framework::lint_rule_test!(DefaultCaseLast);

    #[test]
    fn test_flags_default_not_last() {
        let diags = lint("switch(x) { case 1: break; default: break; case 2: break; }");
        assert_eq!(
            diags.len(),
            1,
            "default case not at the end should be flagged"
        );
    }

    #[test]
    fn test_flags_default_first() {
        let diags = lint("switch(x) { default: break; case 1: break; case 2: break; }");
        assert_eq!(
            diags.len(),
            1,
            "default case at the beginning should be flagged"
        );
    }

    #[test]
    fn test_allows_default_last() {
        let diags = lint("switch(x) { case 1: break; default: break; }");
        assert!(
            diags.is_empty(),
            "default case at the end should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_default() {
        let diags = lint("switch(x) { case 1: break; case 2: break; }");
        assert!(
            diags.is_empty(),
            "switch without default should not be flagged"
        );
    }

    #[test]
    fn test_allows_only_default() {
        let diags = lint("switch(x) { default: break; }");
        assert!(
            diags.is_empty(),
            "switch with only default case should not be flagged"
        );
    }

    #[test]
    fn test_allows_default_last_of_many() {
        let diags =
            lint("switch(x) { case 1: break; case 2: break; case 3: break; default: break; }");
        assert!(
            diags.is_empty(),
            "default as last of many cases should not be flagged"
        );
    }
}
