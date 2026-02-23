//! Rule: `uninvoked-array-callback` (OXC)
//!
//! Detect passing a function reference to an array method that expects a
//! different number of arguments. For example, `['1', '2'].map(parseInt)`
//! doesn't work as expected because `parseInt` receives the index as the
//! second argument (radix).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Known dangerous combinations of array methods + function references.
const DANGEROUS_CALLBACKS: &[(&str, &str)] = &[
    ("map", "parseInt"),
    ("map", "parseFloat"),
    ("map", "Number"),
    ("forEach", "alert"),
    ("map", "Boolean"),
];

/// Flags potentially problematic function references passed to array methods.
#[derive(Debug)]
pub struct UninvokedArrayCallback;

impl LintRule for UninvokedArrayCallback {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "uninvoked-array-callback".to_owned(),
            description: "Detect problematic function references in array callbacks".to_owned(),
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

        // Get the method name from a member expression
        let method_name = match ctx.node(call.callee) {
            Some(AstNode::StaticMemberExpression(member)) => Some(member.property.as_str()),
            _ => None,
        };

        let Some(method) = method_name else {
            return;
        };

        // Check if the first argument is a known dangerous callback
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        let Some(arg_expr) = ctx.node(*first_arg) else {
            return;
        };

        let callback_name = match arg_expr {
            AstNode::IdentifierReference(id) => Some(id.name.as_str()),
            _ => None,
        };

        let Some(cb_name) = callback_name else {
            return;
        };

        for &(arr_method, func_name) in DANGEROUS_CALLBACKS {
            if method == arr_method && cb_name == func_name {
                // Build a wrapping arrow function as a suggestion fix.
                // parseInt gets a radix argument; others just forward the value.
                let arg_span = ctx.node(*first_arg).map_or(Span::new(0, 0), |n| {
                    let s = n.span();
                    Span::new(s.start, s.end)
                });
                let wrapper = if func_name == "parseInt" {
                    format!("(x) => {func_name}(x, 10)")
                } else {
                    format!("(x) => {func_name}(x)")
                };

                ctx.report(Diagnostic {
                    rule_name: "uninvoked-array-callback".to_owned(),
                    message: format!(
                        "Passing `{func_name}` directly to `.{arr_method}()` may produce \
                         unexpected results — the callback receives extra arguments (index, array)"
                    ),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: Some(Fix {
                        kind: FixKind::SuggestionFix,
                        message: format!("Wrap `{func_name}` in an arrow function"),
                        edits: vec![Edit {
                            span: Span::new(arg_span.start, arg_span.end),
                            replacement: wrapper,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(UninvokedArrayCallback)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_map_parse_int() {
        let diags = lint("['1', '2', '3'].map(parseInt);");
        assert_eq!(diags.len(), 1, "map(parseInt) should be flagged");
    }

    #[test]
    fn test_allows_map_with_arrow() {
        let diags = lint("['1', '2', '3'].map(x => parseInt(x, 10));");
        assert!(
            diags.is_empty(),
            "map with arrow function should not be flagged"
        );
    }

    #[test]
    fn test_allows_map_with_custom_function() {
        let diags = lint("[1, 2, 3].map(double);");
        assert!(
            diags.is_empty(),
            "map with custom function should not be flagged"
        );
    }
}
