//! Rule: `bad-array-method-on-arguments` (OXC)
//!
//! Detect calling array methods on the `arguments` object. The `arguments`
//! object is not a real array, so methods like `.map()`, `.filter()`, etc.
//! will fail at runtime.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Array methods that don't exist on the `arguments` object.
const ARRAY_METHODS: &[&str] = &[
    "map",
    "filter",
    "reduce",
    "reduceRight",
    "forEach",
    "some",
    "every",
    "find",
    "findIndex",
    "flat",
    "flatMap",
    "includes",
    "indexOf",
    "lastIndexOf",
    "fill",
    "copyWithin",
    "entries",
    "keys",
    "values",
    "from",
];

/// Flags array methods called on `arguments`.
#[derive(Debug)]
pub struct BadArrayMethodOnArguments;

impl LintRule for BadArrayMethodOnArguments {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "bad-array-method-on-arguments".to_owned(),
            description: "Detect array methods called on `arguments`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        // Check if the object is `arguments`
        let is_arguments = matches!(
            ctx.node(member.object),
            Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "arguments"
        );

        if !is_arguments {
            return;
        }

        let method = member.property.as_str();
        if ARRAY_METHODS.contains(&method) {
            // Fix: arguments.method(...) → Array.from(arguments).method(...)
            #[allow(clippy::as_conversions)]
            let fix = {
                let source = ctx.source_text();
                let call_start = call.span.start as usize;
                let call_end = call.span.end as usize;
                let call_text = source.get(call_start..call_end);
                call_text.map(|text| {
                    let replacement = text.replacen("arguments.", "Array.from(arguments).", 1);
                    Fix {
                        kind: FixKind::SuggestionFix,
                        message: format!("Replace with `Array.from(arguments).{method}()`"),
                        edits: vec![Edit {
                            span: Span::new(call.span.start, call.span.end),
                            replacement,
                        }],
                        is_snippet: false,
                    }
                })
            };

            ctx.report(Diagnostic {
                rule_name: "bad-array-method-on-arguments".to_owned(),
                message: format!(
                    "`arguments.{method}()` will fail — `arguments` is not an array. \
                     Use `Array.from(arguments).{method}()` or rest parameters instead"
                ),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Error,
                help: None,
                fix,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(BadArrayMethodOnArguments)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_arguments_map() {
        let diags = lint("function f() { arguments.map(x => x); }");
        assert_eq!(diags.len(), 1, "arguments.map() should be flagged");
    }

    #[test]
    fn test_flags_arguments_filter() {
        let diags = lint("function f() { arguments.filter(Boolean); }");
        assert_eq!(diags.len(), 1, "arguments.filter() should be flagged");
    }

    #[test]
    fn test_allows_arguments_length() {
        let diags = lint("function f() { return arguments.length; }");
        assert!(diags.is_empty(), "arguments.length should not be flagged");
    }

    #[test]
    fn test_allows_array_map() {
        let diags = lint("var result = arr.map(x => x);");
        assert!(diags.is_empty(), "normal array.map should not be flagged");
    }
}
