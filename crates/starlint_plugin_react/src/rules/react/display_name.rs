//! Rule: `react/display-name`
//!
//! Component definition is missing display name. Components wrapped in
//! `React.memo()` or `React.forwardRef()` with anonymous functions make
//! debugging harder because they appear as "Anonymous" in React `DevTools`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `React.memo()` and `React.forwardRef()` calls with anonymous functions.
#[derive(Debug)]
pub struct DisplayName;

impl LintRule for DisplayName {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/display-name".to_owned(),
            description: "Component definition is missing display name".to_owned(),
            category: Category::Suggestion,
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

        // Check for React.memo(...) or React.forwardRef(...) or memo(...) or forwardRef(...)
        let wrapper_name = match ctx.node(call.callee) {
            Some(AstNode::StaticMemberExpression(member)) => {
                let prop = member.property.as_str();
                (prop == "memo" || prop == "forwardRef").then_some(prop)
            }
            Some(AstNode::IdentifierReference(ident)) => {
                let name = ident.name.as_str();
                (name == "memo" || name == "forwardRef").then_some(name)
            }
            _ => None,
        };

        let Some(wrapper) = wrapper_name else {
            return;
        };

        // Check if the first argument is an anonymous function
        let Some(first_arg_id) = call.arguments.first() else {
            return;
        };

        let is_anonymous = match ctx.node(*first_arg_id) {
            Some(AstNode::Function(func)) => func.id.is_none(),
            Some(AstNode::ArrowFunctionExpression(_)) => true,
            _ => false,
        };

        if is_anonymous {
            ctx.report(Diagnostic {
                rule_name: "react/display-name".to_owned(),
                message: format!("Component wrapped in `{wrapper}` is missing a display name"),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(DisplayName)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_memo_with_arrow() {
        let source = "const Comp = React.memo(() => <div />);";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "React.memo with arrow function should be flagged"
        );
    }

    #[test]
    fn test_flags_forward_ref_with_arrow() {
        let source = "const Comp = React.forwardRef((props, ref) => <div />);";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "React.forwardRef with arrow function should be flagged"
        );
    }

    #[test]
    fn test_flags_memo_with_anonymous_function() {
        let source = "const Comp = React.memo(function() { return <div />; });";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "React.memo with anonymous function should be flagged"
        );
    }

    #[test]
    fn test_allows_memo_with_named_function() {
        let source = "const Comp = React.memo(function MyComp() { return <div />; });";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "React.memo with named function should not be flagged"
        );
    }

    #[test]
    fn test_allows_other_calls() {
        let source = "const x = someFunc(() => <div />);";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "non-memo/forwardRef calls should not be flagged"
        );
    }
}
