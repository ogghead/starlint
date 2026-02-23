//! Rule: `jest/no-alias-methods`
//!
//! Suggest replacing deprecated Jest matcher aliases with their canonical forms.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-alias-methods";

/// Alias -> canonical method mappings.
const ALIASES: &[(&str, &str)] = &[
    ("toBeCalled", "toHaveBeenCalled"),
    ("toBeCalledWith", "toHaveBeenCalledWith"),
    ("lastCalledWith", "toHaveBeenLastCalledWith"),
    ("nthCalledWith", "toHaveBeenNthCalledWith"),
    ("toReturn", "toHaveReturned"),
    ("toReturnWith", "toHaveReturnedWith"),
    ("lastReturnedWith", "toHaveLastReturnedWith"),
    ("nthReturnedWith", "toHaveNthReturnedWith"),
    ("toReturnTimes", "toHaveReturnedTimes"),
    ("toBeCalledTimes", "toHaveBeenCalledTimes"),
];

/// Flags deprecated Jest matcher aliases.
#[derive(Debug)]
pub struct NoAliasMethods;

impl LintRule for NoAliasMethods {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Replace deprecated Jest matcher aliases with canonical forms".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        source_text.contains("expect(") && crate::is_test_file(file_path)
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Check for member expression calls like `expect(x).toBeCalled()`
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        let method_name = member.property.as_str();

        for (alias, canonical) in ALIASES {
            if method_name == *alias {
                // Compute property span from the member's span.
                // The property name is at the end of the member expression span,
                // right after the dot.
                let prop_len = u32::try_from(alias.len()).unwrap_or(0);
                let prop_end = member.span.end;
                let prop_start = prop_end.saturating_sub(prop_len);
                let prop_span = Span::new(prop_start, prop_end);
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!("Use `{canonical}` instead of deprecated `{alias}`"),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Warning,
                    help: Some(format!("Replace `{alias}` with `{canonical}`")),
                    fix: Some(Fix {
                        kind: FixKind::SafeFix,
                        message: format!("Replace with `{canonical}`"),
                        edits: vec![Edit {
                            span: prop_span,
                            replacement: (*canonical).to_owned(),
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoAliasMethods)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_to_be_called() {
        let diags = lint("expect(fn).toBeCalled();");
        assert_eq!(diags.len(), 1, "`toBeCalled` should be flagged");
    }

    #[test]
    fn test_flags_to_return() {
        let diags = lint("expect(fn).toReturn();");
        assert_eq!(diags.len(), 1, "`toReturn` should be flagged");
    }

    #[test]
    fn test_allows_canonical_method() {
        let diags = lint("expect(fn).toHaveBeenCalled();");
        assert!(diags.is_empty(), "`toHaveBeenCalled` should not be flagged");
    }
}
