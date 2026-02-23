//! Rule: `no-unused-private-class-members`
//!
//! Disallow unused private class members. Private fields and methods that are
//! declared but never used are dead code and should be removed.

use std::collections::{HashMap, HashSet};

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags unused private class members (fields and methods).
#[derive(Debug)]
pub struct NoUnusedPrivateClassMembers;

impl LintRule for NoUnusedPrivateClassMembers {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unused-private-class-members".to_owned(),
            description: "Disallow unused private class members".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::Class])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::Class(class) = node else {
            return;
        };

        // Collect declared private members and their spans
        let mut declared: HashMap<String, Span> = HashMap::new();
        // Collect used private member names
        let mut used: HashSet<String> = HashSet::new();

        for element_id in &class.body {
            match ctx.node(*element_id) {
                Some(AstNode::MethodDefinition(method)) => {
                    let key_id = method.key;
                    let value_id = method.value;
                    // Check if key is a private identifier by looking at source text
                    if let Some(key_node) = ctx.node(key_id) {
                        let key_span = key_node.span();
                        let source = ctx.source_text();
                        if let Some(key_text) =
                            source.get(key_span.start as usize..key_span.end as usize)
                        {
                            if key_text.starts_with('#') {
                                let name = key_text.trim_start_matches('#').to_owned();
                                declared.insert(name, Span::new(key_span.start, key_span.end));
                            }
                        }
                    }
                    // Check method body for private member usage
                    if let Some(AstNode::Function(func)) = ctx.node(value_id) {
                        if let Some(body_id) = func.body {
                            if let Some(body_node) = ctx.node(body_id) {
                                let body_span = body_node.span();
                                collect_private_references_from_source(
                                    ctx.source_text(),
                                    body_span.start,
                                    body_span.end,
                                    &mut used,
                                );
                            }
                        }
                    }
                }
                Some(AstNode::PropertyDefinition(prop)) => {
                    let key_id = prop.key;
                    let value_opt = prop.value;
                    // Check if key is a private identifier by looking at source text
                    if let Some(key_node) = ctx.node(key_id) {
                        let key_span = key_node.span();
                        let source = ctx.source_text();
                        if let Some(key_text) =
                            source.get(key_span.start as usize..key_span.end as usize)
                        {
                            if key_text.starts_with('#') {
                                let name = key_text.trim_start_matches('#').to_owned();
                                declared.insert(name, Span::new(key_span.start, key_span.end));
                            }
                        }
                    }
                    // Check initializer for private member usage via source text
                    if let Some(init_id) = value_opt {
                        if let Some(init_node) = ctx.node(init_id) {
                            let init_span = init_node.span();
                            collect_private_references_from_source(
                                ctx.source_text(),
                                init_span.start,
                                init_span.end,
                                &mut used,
                            );
                        }
                    }
                }
                _ => {}
            }
        }

        // Report declared but unused private members
        for (name, span) in &declared {
            if !used.contains(name) {
                ctx.report(Diagnostic {
                    rule_name: "no-unused-private-class-members".to_owned(),
                    message: format!("Private member `#{name}` is declared but never used"),
                    span: *span,
                    severity: Severity::Error,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
        }
    }
}

/// Collect private member references from source text in a span range.
///
/// This is a simple heuristic — we look for `#identifier` patterns in the
/// source text within the given range.
fn collect_private_references_from_source(
    source: &str,
    start: u32,
    end: u32,
    used: &mut HashSet<String>,
) {
    let start_idx = usize::try_from(start).unwrap_or(0);
    let end_idx = usize::try_from(end).unwrap_or(0);
    let Some(text) = source.get(start_idx..end_idx) else {
        return;
    };

    // Find all #identifier patterns
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if bytes.get(i).copied() == Some(b'#') {
            let name_start = i.saturating_add(1);
            let mut name_end = name_start;
            while name_end < len {
                let ch = bytes.get(name_end).copied().unwrap_or(0);
                if ch.is_ascii_alphanumeric() || ch == b'_' {
                    name_end = name_end.saturating_add(1);
                } else {
                    break;
                }
            }
            if name_end > name_start {
                if let Some(name) = text.get(name_start..name_end) {
                    used.insert(name.to_owned());
                }
            }
            i = name_end;
        } else {
            i = i.saturating_add(1);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUnusedPrivateClassMembers)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_unused_private_field() {
        let diags = lint("class A { #unused = 1; method() { return 2; } }");
        assert_eq!(diags.len(), 1, "unused private field should be flagged");
    }

    #[test]
    fn test_allows_used_private_field() {
        let diags = lint("class A { #x = 1; method() { return this.#x; } }");
        assert!(diags.is_empty(), "used private field should not be flagged");
    }

    #[test]
    fn test_flags_unused_private_method() {
        let diags = lint("class A { #unusedMethod() {} method() { return 1; } }");
        assert_eq!(diags.len(), 1, "unused private method should be flagged");
    }

    #[test]
    fn test_allows_used_private_method() {
        let diags = lint("class A { #helper() { return 1; } method() { return this.#helper(); } }");
        assert!(
            diags.is_empty(),
            "used private method should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_private_members() {
        let diags = lint("class A { x = 1; method() { return this.x; } }");
        assert!(
            diags.is_empty(),
            "class without private members should not be flagged"
        );
    }
}
