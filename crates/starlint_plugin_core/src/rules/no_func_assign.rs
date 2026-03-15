//! Rule: `no-func-assign` (eslint)
//!
//! Disallow reassignment of function declarations. Reassigning a function
//! declaration is almost always a mistake.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};
use starlint_scope::SymbolFlags;

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags reassignment of function declarations.
#[derive(Debug)]
pub struct NoFuncAssign;

impl LintRule for NoFuncAssign {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-func-assign".to_owned(),
            description: "Disallow reassignment of function declarations".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn needs_scope_analysis(&self) -> bool {
        true
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::Function])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::Function(func) = node else {
            return;
        };

        // Only check function declarations (not expressions)
        // A function is a declaration if it has an id and its body is present
        // (function expressions can also have ids, but we check via semantic)
        let Some(id_node_id) = func.id else {
            return;
        };

        let Some(AstNode::BindingIdentifier(id)) = ctx.node(id_node_id) else {
            return;
        };

        let id_name = id.name.clone();
        let id_span = id.span;
        let func_span = func.span;
        let is_async = func.is_async;
        let is_generator = func.is_generator;

        let Some(symbol_id) = ctx.resolve_symbol_id(id_span) else {
            return;
        };

        let Some(scope_data) = ctx.scope_data() else {
            return;
        };

        // Check that this symbol has the Function flag
        let flags = scope_data.symbol_flags(symbol_id);
        if !flags.contains(SymbolFlags::FUNCTION) {
            return;
        }

        // Check if any reference to this symbol is a write
        let has_write = scope_data
            .get_resolved_references(symbol_id)
            .iter()
            .any(|r| r.flags.is_write());

        if has_write {
            // Suggest converting function declaration to a `let` variable
            // with a function expression, making reassignment valid.
            let fix = if !is_async && !is_generator {
                let prefix_span = Span::new(func_span.start, id_span.end);
                let mut builder = FixBuilder::new(
                    format!("Convert to `let {id_name} = function`"),
                    FixKind::SuggestionFix,
                )
                .replace(prefix_span, format!("let {id_name} = function"));
                // Add trailing semicolon if not already present.
                let source = ctx.source_text();
                let func_end = usize::try_from(func_span.end).unwrap_or(0);
                if source.as_bytes().get(func_end) != Some(&b';') {
                    builder = builder.insert_at(func_span.end, ";");
                }
                builder.build()
            } else {
                None
            };

            ctx.report(Diagnostic {
                rule_name: "no-func-assign".to_owned(),
                message: format!(
                    "'{id_name}' is a function declaration and should not be reassigned"
                ),
                span: Span::new(id_span.start, id_span.end),
                severity: Severity::Error,
                help: Some(
                    "Use a variable declaration instead if reassignment is intended".to_owned(),
                ),
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoFuncAssign);

    #[test]
    fn test_flags_function_reassignment() {
        let diags = lint("function foo() {} foo = bar;");
        assert_eq!(
            diags.len(),
            1,
            "reassigning function declaration should be flagged"
        );
    }

    #[test]
    fn test_allows_normal_function() {
        let diags = lint("function foo() {} foo();");
        assert!(diags.is_empty(), "calling function should not be flagged");
    }

    #[test]
    fn test_allows_function_expression() {
        let diags = lint("var foo = function() {}; foo = bar;");
        assert!(
            diags.is_empty(),
            "reassigning function expression should not be flagged"
        );
    }

    #[test]
    fn test_allows_different_name() {
        let diags = lint("function foo() {} bar = baz;");
        assert!(
            diags.is_empty(),
            "assigning different name should not be flagged"
        );
    }
}
