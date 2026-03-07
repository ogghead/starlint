//! Rule: `no-extra-semi`
//!
//! Disallow unnecessary semicolons (empty statements). Extra semicolons are
//! usually the result of a typo or copy-paste error and serve no purpose.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags unnecessary semicolons (empty statements, e.g. `;;`).
#[derive(Debug)]
pub struct NoExtraSemi;

impl LintRule for NoExtraSemi {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-extra-semi".to_owned(),
            description: "Disallow unnecessary semicolons".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::EmptyStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        if let AstNode::EmptyStatement(stmt) = node {
            ctx.report(Diagnostic {
                rule_name: "no-extra-semi".to_owned(),
                message: "Unnecessary semicolon".to_owned(),
                span: Span::new(stmt.span.start, stmt.span.end),
                severity: Severity::Warning,
                help: Some("Remove the extra semicolon".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Remove the extra semicolon".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(stmt.span.start, stmt.span.end),
                        replacement: String::new(),
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
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoExtraSemi)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_extra_semicolon() {
        let diags = lint(";;");
        assert!(!diags.is_empty(), "should flag extra semicolons");
    }

    #[test]
    fn test_allows_necessary_semicolons() {
        let diags = lint("const x = 1;");
        assert!(
            diags.is_empty(),
            "necessary semicolons should not be flagged"
        );
    }

    #[test]
    fn test_fix_removes_semicolon() {
        let diags = lint(";;");
        let first = diags.first();
        assert!(first.is_some(), "should have at least one diagnostic");
        if let Some(diag) = first {
            let fix = diag.fix.as_ref();
            assert!(fix.is_some(), "diagnostic should have a fix");
            if let Some(f) = fix {
                let edit = f.edits.first();
                assert!(edit.is_some(), "fix should have at least one edit");
                if let Some(e) = edit {
                    assert!(
                        e.replacement.is_empty(),
                        "fix replacement should be empty string"
                    );
                }
            }
        }
    }
}
