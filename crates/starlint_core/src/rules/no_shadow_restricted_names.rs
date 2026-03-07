//! Rule: `no-shadow-restricted-names`
//!
//! Disallow identifiers from shadowing restricted names such as `undefined`,
//! `NaN`, `Infinity`, `eval`, and `arguments`. Shadowing these can lead to
//! confusing behavior and subtle bugs.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Names that should not be shadowed.
const RESTRICTED_NAMES: &[&str] = &["undefined", "NaN", "Infinity", "eval", "arguments"];

/// Flags binding identifiers that shadow restricted global names.
#[derive(Debug)]
pub struct NoShadowRestrictedNames;

impl LintRule for NoShadowRestrictedNames {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-shadow-restricted-names".to_owned(),
            description: "Disallow identifiers from shadowing restricted names".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BindingIdentifier])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::BindingIdentifier(ident) = node else {
            return;
        };

        let name = ident.name.as_str();
        if RESTRICTED_NAMES.contains(&name) {
            ctx.report(Diagnostic {
                rule_name: "no-shadow-restricted-names".to_owned(),
                message: format!("Shadowing of global property `{name}`"),
                span: Span::new(ident.span.start, ident.span.end),
                severity: Severity::Error,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoShadowRestrictedNames)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_var_undefined() {
        let diags = lint("var undefined = 1;");
        assert_eq!(diags.len(), 1, "shadowing undefined should be flagged");
    }

    #[test]
    fn test_flags_let_nan() {
        let diags = lint("let NaN = 42;");
        assert_eq!(diags.len(), 1, "shadowing NaN should be flagged");
    }

    #[test]
    fn test_flags_const_infinity() {
        let diags = lint("const Infinity = 0;");
        assert_eq!(diags.len(), 1, "shadowing Infinity should be flagged");
    }

    #[test]
    fn test_flags_function_param_eval() {
        let diags = lint("function f(eval) {}");
        assert_eq!(diags.len(), 1, "shadowing eval in params should be flagged");
    }

    #[test]
    fn test_flags_catch_arguments() {
        let diags = lint("try {} catch (arguments) {}");
        assert_eq!(
            diags.len(),
            1,
            "shadowing arguments in catch should be flagged"
        );
    }

    #[test]
    fn test_allows_normal_names() {
        let diags = lint("let x = 1; const y = 2; var z = 3;");
        assert!(diags.is_empty(), "normal names should not be flagged");
    }

    #[test]
    fn test_allows_using_restricted_names() {
        let diags = lint("console.log(undefined, NaN, Infinity);");
        assert!(
            diags.is_empty(),
            "using (not shadowing) restricted names should not be flagged"
        );
    }
}
