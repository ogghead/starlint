//! Rule: `no-class-assign` (eslint)
//!
//! Disallow reassignment of class declarations. Reassigning a class
//! name is almost always a mistake.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};
use starlint_scope::SymbolFlags;

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags reassignment of class declarations.
#[derive(Debug)]
pub struct NoClassAssign;

impl LintRule for NoClassAssign {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-class-assign".to_owned(),
            description: "Disallow reassignment of class declarations".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn needs_scope_analysis(&self) -> bool {
        true
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::Class])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::Class(class) = node else {
            return;
        };

        // Only check class declarations (not expressions)
        // A class is a declaration if it appears as a top-level statement
        // We check by resolving the id and verifying it has the Class symbol flag
        let Some(id_node_id) = class.id else {
            return;
        };

        let Some(AstNode::BindingIdentifier(id)) = ctx.node(id_node_id) else {
            return;
        };

        let id_name = id.name.clone();
        let id_span = id.span;

        let Some(symbol_id) = ctx.resolve_symbol_id(id_span) else {
            return;
        };

        let Some(scope_data) = ctx.scope_data() else {
            return;
        };

        // Check that this symbol has the Class flag
        let flags = scope_data.symbol_flags(symbol_id);
        if !flags.contains(SymbolFlags::CLASS) {
            return;
        }

        // Check if any reference to this symbol is a write
        let has_write = scope_data
            .get_resolved_references(symbol_id)
            .iter()
            .any(|r| r.flags.is_write());

        if has_write {
            // Suggest converting class declaration to a `let` variable
            // with a class expression, making reassignment valid.
            let fix = {
                let prefix_span = Span::new(class.span.start, id_span.end);
                let mut builder = FixBuilder::new(
                    format!("Convert to `let {id_name} = class`"),
                    FixKind::SuggestionFix,
                )
                .replace(prefix_span, format!("let {id_name} = class"));
                // Add trailing semicolon if not already present.
                let source = ctx.source_text();
                let class_end = usize::try_from(class.span.end).unwrap_or(0);
                if source.as_bytes().get(class_end) != Some(&b';') {
                    builder = builder.insert_at(class.span.end, ";");
                }
                builder.build()
            };

            ctx.report(Diagnostic {
                rule_name: "no-class-assign".to_owned(),
                message: format!("'{id_name}' is a class declaration and should not be reassigned"),
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

    starlint_rule_framework::lint_rule_test!(NoClassAssign);

    #[test]
    fn test_flags_class_reassignment() {
        let diags = lint("class Foo {} Foo = bar;");
        assert_eq!(
            diags.len(),
            1,
            "reassigning class declaration should be flagged"
        );
    }

    #[test]
    fn test_allows_class_instantiation() {
        let diags = lint("class Foo {} new Foo();");
        assert!(
            diags.is_empty(),
            "instantiating class should not be flagged"
        );
    }

    #[test]
    fn test_allows_class_expression_reassignment() {
        let diags = lint("var Foo = class {}; Foo = bar;");
        assert!(
            diags.is_empty(),
            "reassigning class expression should not be flagged by this rule"
        );
    }

    #[test]
    fn test_allows_different_name() {
        let diags = lint("class Foo {} bar = baz;");
        assert!(
            diags.is_empty(),
            "assigning different name should not be flagged"
        );
    }
}
