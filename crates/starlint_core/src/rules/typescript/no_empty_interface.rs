//! Rule: `typescript/no-empty-interface`
//!
//! Disallow empty interfaces. An empty `interface` with no members and no
//! `extends` clause is equivalent to `{}` (the empty object type) and is
//! almost always a mistake or a leftover from refactoring.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `interface` declarations with no members and no `extends` clause.
#[derive(Debug)]
pub struct NoEmptyInterface;

impl LintRule for NoEmptyInterface {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-empty-interface".to_owned(),
            description: "Disallow empty interfaces".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TSInterfaceDeclaration])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::TSInterfaceDeclaration(decl) = node else {
            return;
        };

        // Interfaces that extend another type are intentional even if empty
        // (e.g. branding patterns, module augmentation).
        // Check source text for `extends` keyword since the AST doesn't track it.
        let source = ctx.source_text();
        let decl_text = source
            .get(decl.span.start as usize..decl.span.end as usize)
            .unwrap_or("");
        if decl_text.contains("extends") {
            return;
        }

        if decl.body.is_empty() {
            // Resolve the interface name from the id NodeId
            let name = ctx
                .node(decl.id)
                .and_then(|n| n.as_binding_identifier())
                .map_or("<unknown>", |id| id.name.as_str());

            let replacement = format!("type {name} = {{}}");

            let decl_span_start = decl.span.start;
            let decl_span_end = decl.span.end;

            ctx.report(Diagnostic {
                rule_name: "typescript/no-empty-interface".to_owned(),
                message:
                    "Empty interface is equivalent to `{}` — consider removing it or adding members"
                        .to_owned(),
                span: Span::new(decl_span_start, decl_span_end),
                severity: Severity::Warning,
                help: Some("Convert to a type alias".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: format!("Convert to `{replacement}`"),
                    edits: vec![Edit {
                        span: Span::new(decl_span_start, decl_span_end),
                        replacement,
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoEmptyInterface)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_empty_interface() {
        let diags = lint("interface Foo {}");
        assert_eq!(diags.len(), 1, "empty interface should be flagged");
    }

    #[test]
    fn test_allows_interface_with_members() {
        let diags = lint("interface Foo { x: number }");
        assert!(
            diags.is_empty(),
            "interface with members should not be flagged"
        );
    }

    #[test]
    fn test_allows_empty_interface_with_extends() {
        let diags = lint("interface Foo extends Bar {}");
        assert!(
            diags.is_empty(),
            "empty interface with extends should not be flagged"
        );
    }
}
