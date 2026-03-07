//! Rule: `symbol-description`
//!
//! Require a description when creating a `Symbol`. Providing a description
//! makes debugging easier since it appears in `toString()`.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags `Symbol()` calls without a description argument.
#[derive(Debug)]
pub struct SymbolDescription;

impl LintRule for SymbolDescription {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "symbol-description".to_owned(),
            description: "Require a description when creating a Symbol".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        let is_symbol = matches!(
            ctx.node(call.callee),
            Some(AstNode::IdentifierReference(id)) if id.name == "Symbol"
        );

        if !is_symbol {
            return;
        }

        if call.arguments.is_empty() {
            // Fix: Symbol() → Symbol('') — insert empty description
            let fix = {
                let source = ctx.source_text();
                source
                    .get(call.span.start as usize..call.span.end as usize)
                    .and_then(|text| {
                        text.rfind(')').map(|paren_pos| {
                            let insert_pos = call
                                .span
                                .start
                                .saturating_add(u32::try_from(paren_pos).unwrap_or(0));
                            Fix {
                                kind: FixKind::SuggestionFix,
                                message: "Add empty description `''`".to_owned(),
                                edits: vec![Edit {
                                    span: Span::new(insert_pos, insert_pos),
                                    replacement: "''".to_owned(),
                                }],
                                is_snippet: false,
                            }
                        })
                    })
            };

            ctx.report(Diagnostic {
                rule_name: "symbol-description".to_owned(),
                message: "Provide a description for `Symbol()` to aid debugging".to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
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
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(SymbolDescription)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_symbol_without_description() {
        let diags = lint("var s = Symbol();");
        assert_eq!(
            diags.len(),
            1,
            "Symbol() without description should be flagged"
        );
    }

    #[test]
    fn test_allows_symbol_with_description() {
        let diags = lint("var s = Symbol('mySymbol');");
        assert!(
            diags.is_empty(),
            "Symbol with description should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_symbol_call() {
        let diags = lint("var x = foo();");
        assert!(diags.is_empty(), "non-Symbol call should not be flagged");
    }
}
