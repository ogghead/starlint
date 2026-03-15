//! Rule: `react/exhaustive-deps`
//!
//! Warn about missing dependency arrays in React hooks.
//! Simplified: flags when `useEffect`, `useCallback`, or `useMemo` is called
//! without a dependency array (second argument).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags calls to `useEffect`, `useCallback`, or `useMemo` that are missing
/// their dependency array argument (second parameter).
#[derive(Debug)]
pub struct ExhaustiveDeps;

/// Hook names that require a dependency array as their second argument.
const HOOKS_WITH_DEPS: &[&str] = &["useEffect", "useCallback", "useMemo"];

impl LintRule for ExhaustiveDeps {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/exhaustive-deps".to_owned(),
            description: "Warn about missing dependency arrays in hooks".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        let hook_name = match ctx.node(call.callee) {
            Some(AstNode::IdentifierReference(id)) => id.name.clone(),
            _ => return,
        };

        if !HOOKS_WITH_DEPS.contains(&hook_name.as_str()) {
            return;
        }

        // These hooks require at least 2 arguments: the callback and the dependency array
        if call.arguments.len() < 2 {
            ctx.report(Diagnostic {
                rule_name: "react/exhaustive-deps".to_owned(),
                message: format!(
                    "`{hook_name}` is missing its dependency array — this will run on every render"
                ),
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

    starlint_rule_framework::lint_rule_test!(ExhaustiveDeps);

    #[test]
    fn test_flags_use_effect_without_deps() {
        let source = "useEffect(() => { console.log('hi'); });";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "useEffect without deps should be flagged");
    }

    #[test]
    fn test_allows_use_effect_with_deps() {
        let source = "useEffect(() => { console.log('hi'); }, []);";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "useEffect with deps should not be flagged"
        );
    }

    #[test]
    fn test_flags_use_callback_without_deps() {
        let source = "const fn = useCallback(() => {});";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "useCallback without deps should be flagged");
    }

    #[test]
    fn test_allows_use_memo_with_deps() {
        let source = "const val = useMemo(() => compute(), [compute]);";
        let diags = lint(source);
        assert!(diags.is_empty(), "useMemo with deps should not be flagged");
    }
}
