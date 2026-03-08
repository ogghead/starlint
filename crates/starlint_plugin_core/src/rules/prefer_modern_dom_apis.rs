//! Rule: `prefer-modern-dom-apis`
//!
//! Prefer modern DOM APIs over older ones. Flags `insertBefore`,
//! `replaceChild`, `removeChild`, and `appendChild` in favor of their
//! modern replacements: `before`/`after`, `replaceWith`, `remove`, and `append`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags legacy DOM mutation methods in favor of modern alternatives.
#[derive(Debug)]
pub struct PreferModernDomApis;

/// Legacy DOM mutation methods and their modern replacements.
const LEGACY_METHODS: &[(&str, &str)] = &[
    ("insertBefore", "before/after"),
    ("replaceChild", "replaceWith"),
    ("removeChild", "remove"),
    ("appendChild", "append"),
];

impl LintRule for PreferModernDomApis {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-modern-dom-apis".to_owned(),
            description: "Prefer modern DOM APIs over legacy mutation methods".to_owned(),
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

        let method_name = member.property.as_str();

        let Some((_legacy, modern)) = LEGACY_METHODS
            .iter()
            .find(|(legacy, _)| *legacy == method_name)
        else {
            return;
        };

        let obj_id = member.object;

        // Build fix for appendChild(child) -> parent.append(child)
        // and removeChild(child) -> child.remove()
        #[allow(clippy::as_conversions)]
        let fix = {
            let source = ctx.source_text();
            let Some(obj_node) = ctx.node(obj_id) else {
                return;
            };
            let obj_span = obj_node.span();
            let obj_text = source
                .get(obj_span.start as usize..obj_span.end as usize)
                .unwrap_or("");
            match method_name {
                "appendChild" if call.arguments.len() == 1 => {
                    call.arguments.first().and_then(|arg_id| {
                        let arg_node = ctx.node(*arg_id)?;
                        let arg_span = arg_node.span();
                        let arg_text =
                            source.get(arg_span.start as usize..arg_span.end as usize)?;
                        let replacement = format!("{obj_text}.append({arg_text})");
                        Some(Fix {
                            kind: FixKind::SuggestionFix,
                            message: format!("Replace with `{replacement}`"),
                            edits: vec![Edit {
                                span: Span::new(call.span.start, call.span.end),
                                replacement,
                            }],
                            is_snippet: false,
                        })
                    })
                }
                "removeChild" if call.arguments.len() == 1 => {
                    call.arguments.first().and_then(|arg_id| {
                        let arg_node = ctx.node(*arg_id)?;
                        let arg_span = arg_node.span();
                        let arg_text =
                            source.get(arg_span.start as usize..arg_span.end as usize)?;
                        let replacement = format!("{arg_text}.remove()");
                        Some(Fix {
                            kind: FixKind::SuggestionFix,
                            message: format!("Replace with `{replacement}`"),
                            edits: vec![Edit {
                                span: Span::new(call.span.start, call.span.end),
                                replacement,
                            }],
                            is_snippet: false,
                        })
                    })
                }
                "replaceChild" if call.arguments.len() == 2 => {
                    call.arguments.first().and_then(|new_arg_id| {
                        let old_arg_id = call.arguments.get(1)?;
                        let new_node = ctx.node(*new_arg_id)?;
                        let old_node = ctx.node(*old_arg_id)?;
                        let new_span = new_node.span();
                        let old_span = old_node.span();
                        let new_text =
                            source.get(new_span.start as usize..new_span.end as usize)?;
                        let old_text =
                            source.get(old_span.start as usize..old_span.end as usize)?;
                        let replacement = format!("{old_text}.replaceWith({new_text})");
                        Some(Fix {
                            kind: FixKind::SuggestionFix,
                            message: format!("Replace with `{replacement}`"),
                            edits: vec![Edit {
                                span: Span::new(call.span.start, call.span.end),
                                replacement,
                            }],
                            is_snippet: false,
                        })
                    })
                }
                _ => None,
            }
        };

        ctx.report(Diagnostic {
            rule_name: "prefer-modern-dom-apis".to_owned(),
            message: format!("Prefer `{modern}` over `{method_name}`"),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some(format!(
                "Use `{modern}` instead of `{method_name}` for cleaner, more readable code"
            )),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferModernDomApis)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_insert_before() {
        let diags = lint("parent.insertBefore(newNode, refNode);");
        assert_eq!(diags.len(), 1, "insertBefore should be flagged");
    }

    #[test]
    fn test_flags_replace_child() {
        let diags = lint("parent.replaceChild(newNode, oldNode);");
        assert_eq!(diags.len(), 1, "replaceChild should be flagged");
    }

    #[test]
    fn test_flags_remove_child() {
        let diags = lint("parent.removeChild(child);");
        assert_eq!(diags.len(), 1, "removeChild should be flagged");
    }

    #[test]
    fn test_flags_append_child() {
        let diags = lint("parent.appendChild(child);");
        assert_eq!(diags.len(), 1, "appendChild should be flagged");
    }

    #[test]
    fn test_allows_remove() {
        let diags = lint("node.remove();");
        assert!(diags.is_empty(), "remove() should not be flagged");
    }

    #[test]
    fn test_allows_append() {
        let diags = lint("parent.append(child);");
        assert!(diags.is_empty(), "append() should not be flagged");
    }

    #[test]
    fn test_allows_before() {
        let diags = lint("node.before(newNode);");
        assert!(diags.is_empty(), "before() should not be flagged");
    }

    #[test]
    fn test_allows_replace_with() {
        let diags = lint("node.replaceWith(newNode);");
        assert!(diags.is_empty(), "replaceWith() should not be flagged");
    }

    #[test]
    fn test_allows_unrelated_method() {
        let diags = lint("parent.contains(child);");
        assert!(
            diags.is_empty(),
            "unrelated DOM methods should not be flagged"
        );
    }
}
