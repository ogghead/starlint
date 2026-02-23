//! Rule: `react/forward-ref-uses-ref`
//!
//! Warn when `React.forwardRef()` is used but the `ref` parameter is not used.

#![allow(clippy::indexing_slicing)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `React.forwardRef()` calls where the callback's second parameter
/// (the `ref`) is missing or unused. If `forwardRef` is used but the ref
/// is not forwarded, it is likely a mistake.
#[derive(Debug)]
pub struct ForwardRefUsesRef;

/// Check whether the callee is `React.forwardRef` or just `forwardRef`.
fn is_forward_ref(callee_id: NodeId, ctx: &LintContext<'_>) -> bool {
    match ctx.node(callee_id) {
        Some(AstNode::StaticMemberExpression(member)) => {
            member.property.as_str() == "forwardRef"
                && matches!(
                    ctx.node(member.object),
                    Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "React"
                )
        }
        Some(AstNode::IdentifierReference(id)) => id.name.as_str() == "forwardRef",
        _ => false,
    }
}

/// Check if a function's parameter list has at least 2 parameters, and the second
/// one is actually named (not just `_` which is conventionally unused).
fn ref_param_is_used(params: &[NodeId], source: &str, ctx: &LintContext<'_>) -> bool {
    if params.len() < 2 {
        return false;
    }
    let Some(ref_param_node) = ctx.node(params[1]) else {
        return false;
    };
    let span = ref_param_node.span();
    let Ok(start) = usize::try_from(span.start) else {
        return false;
    };
    let Ok(end) = usize::try_from(span.end) else {
        return false;
    };
    if end > source.len() {
        return false;
    }
    let param_text = &source[start..end];
    let name = param_text.trim();
    // If the parameter is exactly `_`, it's unused by convention
    name != "_"
}

impl LintRule for ForwardRefUsesRef {
    fn should_run_on_file(&self, source_text: &str, _file_path: &std::path::Path) -> bool {
        source_text.contains("forwardRef")
    }

    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/forward-ref-uses-ref".to_owned(),
            description: "Warn when forwardRef is used but ref parameter is not used".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::indexing_slicing)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        if !is_forward_ref(call.callee, ctx) {
            return;
        }

        // forwardRef takes one argument: a callback (props, ref) => ...
        let Some(&first_arg_id) = call.arguments.first() else {
            return;
        };

        let params_missing_ref = match ctx.node(first_arg_id) {
            Some(AstNode::ArrowFunctionExpression(arrow)) => {
                !ref_param_is_used(&arrow.params, ctx.source_text(), ctx)
            }
            Some(AstNode::Function(func)) => {
                !ref_param_is_used(&func.params, ctx.source_text(), ctx)
            }
            _ => false,
        };

        if params_missing_ref {
            ctx.report(Diagnostic {
                rule_name: "react/forward-ref-uses-ref".to_owned(),
                message: "`forwardRef` is used but the `ref` parameter is missing or unused"
                    .to_owned(),
                span: Span::new(call.span.start, call.span.end),
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
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ForwardRefUsesRef)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_forward_ref_without_ref_param() {
        let source = "const Comp = React.forwardRef((props) => <div />);";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "forwardRef without ref parameter should be flagged"
        );
    }

    #[test]
    fn test_allows_forward_ref_with_ref_param() {
        let source = "const Comp = React.forwardRef((props, ref) => <div ref={ref} />);";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "forwardRef with ref parameter should not be flagged"
        );
    }

    #[test]
    fn test_flags_forward_ref_with_unused_ref() {
        let source = "const Comp = forwardRef((props, _) => <div />);";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "forwardRef with underscore ref should be flagged"
        );
    }
}
