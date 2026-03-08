//! Rule: `no-array-reduce`
//!
//! Disallow `Array#reduce()` and `Array#reduceRight()`. These methods
//! often produce hard-to-read code. Prefer `for...of` loops or other
//! array methods for better readability.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags calls to `.reduce()` and `.reduceRight()`.
#[derive(Debug)]
pub struct NoArrayReduce;

impl LintRule for NoArrayReduce {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-array-reduce".to_owned(),
            description: "Disallow Array#reduce() and Array#reduceRight()".to_owned(),
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

        let method = member.property.as_str();
        if method != "reduce" && method != "reduceRight" {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "no-array-reduce".to_owned(),
            message: format!("Prefer `for...of` or other array methods over `.{method}()`"),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoArrayReduce)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_reduce() {
        let diags = lint("arr.reduce((acc, x) => acc + x, 0);");
        assert_eq!(diags.len(), 1, ".reduce() should be flagged");
    }

    #[test]
    fn test_flags_reduce_right() {
        let diags = lint("arr.reduceRight((acc, x) => acc + x, 0);");
        assert_eq!(diags.len(), 1, ".reduceRight() should be flagged");
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
    fn test_allows_for_each() {
        let diags = lint("arr.forEach(x => console.log(x));");
        assert!(diags.is_empty(), ".forEach() should not be flagged");
    }
}
