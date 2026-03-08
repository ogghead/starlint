//! Rule: `typescript/prefer-enum-initializers`
//!
//! Require explicit initializers for all enum members. When enum members rely
//! on implicit auto-incrementing values, inserting or removing a member can
//! silently change the values of subsequent members, leading to subtle bugs
//! (e.g. serialized values no longer matching, switch cases breaking).
//! Requiring explicit initializers makes the intent clear and prevents
//! accidental value drift.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags enum members that lack an explicit initializer.
#[derive(Debug)]
pub struct PreferEnumInitializers;

impl LintRule for PreferEnumInitializers {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/prefer-enum-initializers".to_owned(),
            description: "Require explicit initializers for enum members".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TSEnumDeclaration])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::TSEnumDeclaration(decl) = node else {
            return;
        };

        let mut index: u32 = 0;
        for member_id in &decl.members {
            let Some(AstNode::TSEnumMember(member)) = ctx.node(*member_id) else {
                index = index.saturating_add(1);
                continue;
            };

            if member.initializer.is_none() {
                // Resolve the member name from the id NodeId
                let member_name = ctx.node(member.id).map_or("<unknown>", |n| {
                    let s = n.span();
                    ctx.source_text()
                        .get(s.start as usize..s.end as usize)
                        .unwrap_or("<unknown>")
                });

                // Extract spans before calling ctx.report()
                let member_span_start = member.span.start;
                let member_span_end = member.span.end;
                let member_name_owned = member_name.to_owned();

                // Insert ` = <index>` right after the member identifier (at end of member span)
                let fix = Some(Fix {
                    kind: FixKind::SafeFix,
                    message: format!("Add initializer `= {index}`"),
                    edits: vec![Edit {
                        span: Span::new(member_span_end, member_span_end),
                        replacement: format!(" = {index}"),
                    }],
                    is_snippet: false,
                });

                ctx.report(Diagnostic {
                    rule_name: "typescript/prefer-enum-initializers".to_owned(),
                    message: format!(
                        "Enum member `{member_name_owned}` should have an explicit initializer"
                    ),
                    span: Span::new(member_span_start, member_span_end),
                    severity: Severity::Warning,
                    help: Some(format!("Add `= {index}` to `{member_name_owned}`")),
                    fix,
                    labels: vec![],
                });
            }
            index = index.saturating_add(1);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferEnumInitializers)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_member_without_initializer() {
        let diags = lint("enum Direction { Up, Down }");
        assert_eq!(
            diags.len(),
            2,
            "both enum members without initializers should be flagged"
        );
    }

    #[test]
    fn test_flags_mixed_members() {
        let diags = lint("enum E { A = 0, B, C = 2 }");
        assert_eq!(
            diags.len(),
            1,
            "only the member without an initializer should be flagged"
        );
    }

    #[test]
    fn test_allows_all_initialized_numeric() {
        let diags = lint("enum Direction { Up = 0, Down = 1, Left = 2, Right = 3 }");
        assert!(
            diags.is_empty(),
            "enum with all numeric initializers should not be flagged"
        );
    }

    #[test]
    fn test_allows_all_initialized_string() {
        let diags = lint(r#"enum Color { Red = "RED", Green = "GREEN" }"#);
        assert!(
            diags.is_empty(),
            "enum with all string initializers should not be flagged"
        );
    }

    #[test]
    fn test_flags_single_uninitialized_member() {
        let diags = lint("enum E { A }");
        assert_eq!(
            diags.len(),
            1,
            "single enum member without initializer should be flagged"
        );
    }
}
