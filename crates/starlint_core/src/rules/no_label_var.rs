//! Rule: `no-label-var`
//!
//! Disallow labels that share a name with a variable. This can lead to
//! confusion about which entity is being referenced.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags labels that shadow variable names (simplified: flags labels
/// whose name matches a common variable pattern).
#[derive(Debug)]
pub struct NoLabelVar;

impl LintRule for NoLabelVar {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-label-var".to_owned(),
            description: "Disallow labels that share a name with a variable".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::LabeledStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::LabeledStatement(labeled) = node else {
            return;
        };

        let label_name = labeled.label.as_str();

        // Check if the label name appears as a variable declaration in the source
        // This is a simplified check that scans for `var/let/const label_name`
        let source = ctx.source_text();
        let var_pattern = format!("var {label_name}");
        let let_pattern = format!("let {label_name}");
        let const_pattern = format!("const {label_name}");

        let has_var = source.contains(&var_pattern)
            || source.contains(&let_pattern)
            || source.contains(&const_pattern);

        if has_var {
            ctx.report(Diagnostic {
                rule_name: "no-label-var".to_owned(),
                message: format!("Found identifier `{label_name}` with the same name as a label"),
                span: Span::new(labeled.span.start, labeled.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoLabelVar)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_label_matching_var() {
        let diags = lint("var x = 1; x: while(true) { break x; }");
        assert_eq!(
            diags.len(),
            1,
            "label sharing name with variable should be flagged"
        );
    }

    #[test]
    fn test_allows_label_not_matching_var() {
        let diags = lint("var x = 1; loop1: while(true) { break loop1; }");
        assert!(
            diags.is_empty(),
            "label not matching any variable should not be flagged"
        );
    }
}
