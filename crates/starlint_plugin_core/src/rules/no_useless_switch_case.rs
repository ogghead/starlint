//! Rule: `no-useless-switch-case` (unicorn)
//!
//! Disallow useless case in switch statements. A switch with only a
//! default case, or a case that has the same body as the default case
//! (falling through), is unnecessary complexity.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags switch statements where all cases simply fall through to default.
#[derive(Debug)]
pub struct NoUselessSwitchCase;

/// Info about a switch case collected upfront to avoid borrow conflicts.
struct CaseInfo {
    /// Case span.
    span: Span,
    /// Whether this is the `default:` case.
    is_default: bool,
    /// Whether the consequent is empty.
    consequent_is_empty: bool,
    /// Start/end offsets of the first and last consequent statements.
    consequent_range: Option<(u32, u32)>,
}

impl LintRule for NoUselessSwitchCase {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-switch-case".to_owned(),
            description: "Disallow useless case in switch statements".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::SwitchStatement])
    }

    #[allow(clippy::too_many_lines)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::SwitchStatement(switch) = node else {
            return;
        };

        // Collect case info upfront to avoid borrow conflicts with ctx
        let cases_info: Vec<CaseInfo> = switch
            .cases
            .iter()
            .filter_map(|&case_id| {
                let case = ctx.node(case_id)?.as_switch_case()?;
                let consequent_range = if case.consequent.is_empty() {
                    None
                } else {
                    let first_start = case
                        .consequent
                        .first()
                        .and_then(|&id| ctx.node(id))
                        .map(|n| n.span().start);
                    let last_end = case
                        .consequent
                        .last()
                        .and_then(|&id| ctx.node(id))
                        .map(|n| n.span().end);
                    match (first_start, last_end) {
                        (Some(s), Some(e)) => Some((s, e)),
                        _ => None,
                    }
                };
                Some(CaseInfo {
                    span: Span::new(case.span.start, case.span.end),
                    is_default: case.test.is_none(),
                    consequent_is_empty: case.consequent.is_empty(),
                    consequent_range,
                })
            })
            .collect();

        // Find the default case
        let has_default = cases_info.iter().any(|c| c.is_default);
        if !has_default {
            return;
        }

        let case_count = cases_info.len();

        for (i, case) in cases_info.iter().enumerate() {
            // Skip the default case itself
            if case.is_default {
                continue;
            }

            // If this case has an empty consequent and the next case is default,
            // it's useless — it just falls through to default
            if case.consequent_is_empty {
                let next_is_default = cases_info
                    .get(i.saturating_add(1))
                    .is_some_and(|next| next.is_default);

                if next_is_default {
                    let fix = Some(Fix {
                        kind: FixKind::SafeFix,
                        message: "Remove useless case clause".to_owned(),
                        edits: vec![Edit {
                            span: case.span,
                            replacement: String::new(),
                        }],
                        is_snippet: false,
                    });

                    ctx.report(Diagnostic {
                        rule_name: "no-useless-switch-case".to_owned(),
                        message: "Useless case — falls through to default".to_owned(),
                        span: case.span,
                        severity: Severity::Warning,
                        help: None,
                        fix,
                        labels: vec![],
                    });
                }
            }

            // If it's the only non-default case and has the same sole
            // body as default, flag it.
            if case_count == 2 && !case.consequent_is_empty {
                let default_case = cases_info.iter().find(|c| c.is_default);
                if let Some(dc) = default_case {
                    if !dc.consequent_is_empty {
                        if let (Some(case_range), Some(dc_range)) =
                            (case.consequent_range, dc.consequent_range)
                        {
                            let source = ctx.source_text();
                            let case_body = get_text(source, case_range.0, case_range.1);
                            let default_body = get_text(source, dc_range.0, dc_range.1);
                            if case_body == default_body && !case_body.is_empty() {
                                let fix = Some(Fix {
                                    kind: FixKind::SafeFix,
                                    message: "Remove duplicate case clause".to_owned(),
                                    edits: vec![Edit {
                                        span: case.span,
                                        replacement: String::new(),
                                    }],
                                    is_snippet: false,
                                });

                                ctx.report(Diagnostic {
                                    rule_name: "no-useless-switch-case".to_owned(),
                                    message: "Useless case — has the same body as default"
                                        .to_owned(),
                                    span: case.span,
                                    severity: Severity::Warning,
                                    help: None,
                                    fix,
                                    labels: vec![],
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Extract trimmed text from source by u32 offsets.
fn get_text(source: &str, start: u32, end: u32) -> &str {
    let s = usize::try_from(start).unwrap_or(0);
    let e = usize::try_from(end).unwrap_or(0).min(source.len());
    source.get(s..e).unwrap_or("").trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    starlint_rule_framework::lint_rule_test!(NoUselessSwitchCase);

    #[test]
    fn test_flags_empty_case_before_default() {
        let diags = lint("switch (x) { case 1: default: break; }");
        assert_eq!(
            diags.len(),
            1,
            "empty case falling to default should be flagged"
        );
    }

    #[test]
    fn test_allows_case_with_body() {
        let diags = lint("switch (x) { case 1: foo(); break; default: break; }");
        assert!(
            diags.is_empty(),
            "case with its own body should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_default() {
        let diags = lint("switch (x) { case 1: break; case 2: break; }");
        assert!(
            diags.is_empty(),
            "switch without default should not be flagged"
        );
    }

    #[test]
    fn test_allows_separate_behaviors() {
        let diags = lint(
            "switch (x) { case 1: foo(); break; case 2: bar(); break; default: baz(); break; }",
        );
        assert!(
            diags.is_empty(),
            "cases with different behaviors should not be flagged"
        );
    }
}
