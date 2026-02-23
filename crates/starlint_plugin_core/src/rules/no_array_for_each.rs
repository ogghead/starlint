//! Rule: `no-array-for-each`
//!
//! Disallow `Array#forEach()`. Prefer `for...of` loops for iterating
//! over arrays. `for...of` supports `break`, `continue`, and `await`,
//! and avoids the overhead of a callback function.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags calls to `.forEach()`.
#[derive(Debug)]
pub struct NoArrayForEach;

impl LintRule for NoArrayForEach {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-array-for-each".to_owned(),
            description: "Disallow Array#forEach()".to_owned(),
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

        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property.as_str() != "forEach" {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "no-array-for-each".to_owned(),
            message: "Prefer `for...of` over `.forEach()`".to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoArrayForEach)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_for_each_arrow() {
        let diags = lint("arr.forEach(x => console.log(x));");
        assert_eq!(diags.len(), 1, ".forEach() with arrow should be flagged");
    }

    #[test]
    fn test_flags_for_each_function() {
        let diags = lint("arr.forEach(function(x) { console.log(x); });");
        assert_eq!(
            diags.len(),
            1,
            ".forEach() with function expression should be flagged"
        );
    }

    #[test]
    fn test_allows_for_of() {
        let diags = lint("for (const x of arr) { console.log(x); }");
        assert!(diags.is_empty(), "for...of should not be flagged");
    }

    #[test]
    fn test_allows_map() {
        let diags = lint("arr.map(x => x + 1);");
        assert!(diags.is_empty(), ".map() should not be flagged");
    }

    #[test]
    fn test_allows_filter() {
        let diags = lint("arr.filter(x => x > 0);");
        assert!(diags.is_empty(), ".filter() should not be flagged");
    }

    #[test]
    fn test_allows_reduce() {
        let diags = lint("arr.reduce((acc, x) => acc + x, 0);");
        assert!(diags.is_empty(), ".reduce() should not be flagged");
    }
}
