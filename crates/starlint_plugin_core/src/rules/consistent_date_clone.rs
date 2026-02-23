//! Rule: `consistent-date-clone`
//!
//! Flag `new Date(date.getTime())` — prefer `new Date(date)` for cloning
//! dates. The `getTime()` call is unnecessary when passing to the `Date`
//! constructor.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `new Date(d.getTime())` — prefer `new Date(d)`.
#[derive(Debug)]
pub struct ConsistentDateClone;

impl LintRule for ConsistentDateClone {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "consistent-date-clone".to_owned(),
            description: "Prefer `new Date(date)` over `new Date(date.getTime())`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::NewExpression])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::NewExpression(new_expr) = node else {
            return;
        };

        // Check callee is `Date`
        let is_date = matches!(
            ctx.node(new_expr.callee),
            Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "Date"
        );
        if !is_date {
            return;
        }

        // Must have exactly one argument
        if new_expr.arguments.len() != 1 {
            return;
        }

        let Some(&first_arg_id) = new_expr.arguments.first() else {
            return;
        };

        // The argument must be a call expression (not a spread)
        let Some(AstNode::CallExpression(call)) = ctx.node(first_arg_id) else {
            return;
        };

        // The call must be `.getTime()` with no arguments
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property.as_str() != "getTime" {
            return;
        }

        if !call.arguments.is_empty() {
            return;
        }

        // Fix: replace the argument `d.getTime()` with just `d`
        let source = ctx.source_text();
        let obj_ast_span = ctx.node(member.object).map(starlint_ast::AstNode::span);
        let (obj_start, obj_end) =
            obj_ast_span.map_or((0usize, 0usize), |s| (s.start as usize, s.end as usize));
        let obj_text = source.get(obj_start..obj_end).unwrap_or("").to_owned();

        // Replace the entire first argument (the call expression) with just the object
        let arg_span = call.span;
        let fix = (!obj_text.is_empty()).then(|| Fix {
            kind: FixKind::SafeFix,
            message: format!("Replace `{obj_text}.getTime()` with `{obj_text}`"),
            edits: vec![Edit {
                span: Span::new(arg_span.start, arg_span.end),
                replacement: obj_text.clone(),
            }],
            is_snippet: false,
        });

        ctx.report(Diagnostic {
            rule_name: "consistent-date-clone".to_owned(),
            message: "Use `new Date(date)` instead of `new Date(date.getTime())`".to_owned(),
            span: Span::new(new_expr.span.start, new_expr.span.end),
            severity: Severity::Warning,
            help: Some(format!("Replace `{obj_text}.getTime()` with `{obj_text}`")),
            fix,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ConsistentDateClone)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_date_get_time_clone() {
        let diags = lint("var d2 = new Date(d.getTime());");
        assert_eq!(diags.len(), 1, "new Date(d.getTime()) should be flagged");
    }

    #[test]
    fn test_allows_date_direct_clone() {
        let diags = lint("var d2 = new Date(d);");
        assert!(diags.is_empty(), "new Date(d) should not be flagged");
    }

    #[test]
    fn test_allows_date_no_args() {
        let diags = lint("var d = new Date();");
        assert!(diags.is_empty(), "new Date() should not be flagged");
    }

    #[test]
    fn test_allows_date_multiple_args() {
        let diags = lint("var d = new Date(2024, 0, 1);");
        assert!(diags.is_empty(), "new Date(y, m, d) should not be flagged");
    }

    #[test]
    fn test_allows_non_date_constructor() {
        let diags = lint("var x = new Foo(d.getTime());");
        assert!(
            diags.is_empty(),
            "non-Date constructor should not be flagged"
        );
    }
}
