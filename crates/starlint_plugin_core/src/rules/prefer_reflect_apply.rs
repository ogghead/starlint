//! Rule: `prefer-reflect-apply`
//!
//! Prefer `Reflect.apply()` over `Function.prototype.apply()`. The
//! `Reflect.apply()` method is clearer and avoids relying on `.apply()`
//! being present on the function object.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.apply()` calls with two arguments, suggesting `Reflect.apply()`.
#[derive(Debug)]
pub struct PreferReflectApply;

impl LintRule for PreferReflectApply {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-reflect-apply".to_owned(),
            description: "Prefer `Reflect.apply()` over `Function.prototype.apply()`".to_owned(),
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

        // Must be calling `.apply()`
        if member.property.as_str() != "apply" {
            return;
        }

        // Must have exactly 2 arguments (thisArg, argsArray)
        if call.arguments.len() != 2 {
            return;
        }

        // Skip if the receiver is already `Reflect` (i.e. `Reflect.apply(...)`)
        if let Some(AstNode::IdentifierReference(ident)) = ctx.node(member.object) {
            if ident.name.as_str() == "Reflect" {
                return;
            }
        }

        let source = ctx.source_text();
        let obj_span = ctx.node(member.object).map_or(
            starlint_ast::types::Span::new(0, 0),
            starlint_ast::AstNode::span,
        );
        let fn_start = usize::try_from(obj_span.start).unwrap_or(0);
        let fn_end = usize::try_from(obj_span.end).unwrap_or(0);
        let fn_text = source.get(fn_start..fn_end).unwrap_or("");

        let fix = call.arguments.first().zip(call.arguments.get(1)).and_then(
            |(&ctx_arg_id, &args_arg_id)| {
                let ctx_span = ctx.node(ctx_arg_id)?.span();
                let args_span = ctx.node(args_arg_id)?.span();
                let ctx_start = usize::try_from(ctx_span.start).unwrap_or(0);
                let ctx_end = usize::try_from(ctx_span.end).unwrap_or(0);
                let args_start = usize::try_from(args_span.start).unwrap_or(0);
                let args_end = usize::try_from(args_span.end).unwrap_or(0);
                let ctx_text = source.get(ctx_start..ctx_end).unwrap_or("");
                let args_text = source.get(args_start..args_end).unwrap_or("");
                Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: "Replace with `Reflect.apply()`".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(call.span.start, call.span.end),
                        replacement: format!("Reflect.apply({fn_text}, {ctx_text}, {args_text})"),
                    }],
                    is_snippet: false,
                })
            },
        );

        ctx.report(Diagnostic {
            rule_name: "prefer-reflect-apply".to_owned(),
            message: "Use `Reflect.apply()` instead of `.apply()`".to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some("Replace with `Reflect.apply()`".to_owned()),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferReflectApply)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_apply_with_null() {
        let diags = lint("foo.apply(null, args);");
        assert_eq!(diags.len(), 1, "foo.apply(null, args) should be flagged");
    }

    #[test]
    fn test_flags_apply_with_this_arg() {
        let diags = lint("foo.apply(thisArg, args);");
        assert_eq!(diags.len(), 1, "foo.apply(thisArg, args) should be flagged");
    }

    #[test]
    fn test_allows_apply_with_one_arg() {
        let diags = lint("foo.apply(thisArg);");
        assert!(
            diags.is_empty(),
            "foo.apply(thisArg) with one arg should not be flagged"
        );
    }

    #[test]
    fn test_allows_call() {
        let diags = lint("foo.call(thisArg, a, b);");
        assert!(diags.is_empty(), "foo.call() should not be flagged");
    }

    #[test]
    fn test_allows_reflect_apply() {
        let diags = lint("Reflect.apply(foo, null, args);");
        assert!(diags.is_empty(), "Reflect.apply() should not be flagged");
    }

    #[test]
    fn test_allows_normal_call() {
        let diags = lint("foo(1, 2);");
        assert!(diags.is_empty(), "normal call should not be flagged");
    }
}
