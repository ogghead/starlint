//! Shared JSX attribute utilities for lint rules.
//!
//! Provides helpers for inspecting JSX attributes by name, extracting
//! string values, and querying attribute presence. These are used
//! across multiple plugin crates (react, nextjs, etc.).

use starlint_ast::node::AstNode;
use starlint_ast::types::NodeId;

use crate::LintContext;

/// Check if a JSX element has an attribute with the given `name`.
///
/// Iterates over the `attributes` [`NodeId`] slice (from a
/// `JSXOpeningElement`) and returns `true` if any `JSXAttribute` has a
/// matching name.
#[must_use]
pub fn has_jsx_attribute(attributes: &[NodeId], name: &str, ctx: &LintContext<'_>) -> bool {
    attributes.iter().any(|&attr_id| {
        if let Some(AstNode::JSXAttribute(attr)) = ctx.node(attr_id) {
            attr.name == name
        } else {
            false
        }
    })
}

/// Get the string value of a JSX attribute by name.
///
/// Returns `Some(value)` if the attribute exists and its value is a
/// [`StringLiteral`](AstNode::StringLiteral). Returns `None` if the
/// attribute is missing, has no value, or has a non-string value.
#[must_use]
pub fn get_jsx_attr_string_value(
    attributes: &[NodeId],
    attr_name: &str,
    ctx: &LintContext<'_>,
) -> Option<String> {
    for &attr_id in attributes {
        if let Some(AstNode::JSXAttribute(attr)) = ctx.node(attr_id) {
            if attr.name == attr_name {
                if let Some(value_id) = attr.value {
                    if let Some(AstNode::StringLiteral(lit)) = ctx.node(value_id) {
                        return Some(lit.value.clone());
                    }
                }
            }
        }
    }
    None
}

/// Get the [`NodeId`] of a JSX attribute's value by attribute name.
///
/// Returns `Some(value_id)` if the attribute exists and has a value node.
#[must_use]
pub fn get_jsx_attr_value(
    attributes: &[NodeId],
    attr_name: &str,
    ctx: &LintContext<'_>,
) -> Option<NodeId> {
    for &attr_id in attributes {
        if let Some(AstNode::JSXAttribute(attr)) = ctx.node(attr_id) {
            if attr.name == attr_name {
                return attr.value;
            }
        }
    }
    None
}

/// Extract a string value from a JSX attribute value [`NodeId`].
///
/// Returns `Some(value)` if the node is a [`StringLiteral`](AstNode::StringLiteral).
#[must_use]
pub fn get_string_value(ctx: &LintContext<'_>, value: Option<NodeId>) -> Option<String> {
    let id = value?;
    let node = ctx.node(id)?;
    if let AstNode::StringLiteral(lit) = node {
        Some(lit.value.clone())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use starlint_ast::node::AstNode;
    use starlint_ast::node_type::AstNodeType;
    use starlint_ast::types::NodeId;
    use starlint_parser::ParseOptions;
    use starlint_plugin_sdk::diagnostic::{Severity, Span};
    use starlint_plugin_sdk::rule::{Category, RuleMeta};

    use super::*;
    use crate::lint_rule::LintRule;
    use crate::traversal::{LintDispatchTable, traverse_ast_tree};

    /// Parse and run rules, returning diagnostics.
    fn run_rules(
        source: &str,
        file_path: &str,
        rules: &[Box<dyn LintRule>],
    ) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let path = Path::new(file_path);
        let options = ParseOptions::from_path(path);
        let tree = starlint_parser::parse(source, options).tree;
        let traversal_indices: Vec<usize> = rules
            .iter()
            .enumerate()
            .filter(|(_, r)| r.needs_traversal())
            .map(|(i, _)| i)
            .collect();
        let run_once_indices: Vec<usize> = rules
            .iter()
            .enumerate()
            .filter(|(_, r)| !r.needs_traversal())
            .map(|(i, _)| i)
            .collect();
        let table = LintDispatchTable::build_from_indices(rules, &traversal_indices);
        traverse_ast_tree(&tree, rules, &table, &run_once_indices, source, path, None)
    }

    // ── Test rules that exercise JSX utility functions ──

    /// Rule that checks `has_jsx_attribute` on JSX opening elements.
    #[derive(Debug)]
    struct HasAttrChecker;

    impl LintRule for HasAttrChecker {
        fn meta(&self) -> RuleMeta {
            RuleMeta {
                name: "has-attr".to_owned(),
                description: String::new(),
                category: Category::Correctness,
                default_severity: Severity::Warning,
            }
        }

        fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
            Some(&[AstNodeType::JSXOpeningElement])
        }

        fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
            if let AstNode::JSXOpeningElement(elem) = node {
                if has_jsx_attribute(&elem.attributes, "href", ctx) {
                    ctx.report_warning("has-attr", "has-href", Span::new(0, 0));
                }
                if has_jsx_attribute(&elem.attributes, "target", ctx) {
                    ctx.report_warning("has-attr", "has-target", Span::new(0, 0));
                }
                if !has_jsx_attribute(&elem.attributes, "missing", ctx) {
                    ctx.report_warning("has-attr", "no-missing", Span::new(0, 0));
                }
            }
        }
    }

    /// Rule that checks `get_jsx_attr_string_value`.
    #[derive(Debug)]
    struct AttrStringValueChecker;

    impl LintRule for AttrStringValueChecker {
        fn meta(&self) -> RuleMeta {
            RuleMeta {
                name: "attr-value".to_owned(),
                description: String::new(),
                category: Category::Correctness,
                default_severity: Severity::Warning,
            }
        }

        fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
            Some(&[AstNodeType::JSXOpeningElement])
        }

        fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
            if let AstNode::JSXOpeningElement(elem) = node {
                if let Some(val) = get_jsx_attr_string_value(&elem.attributes, "href", ctx) {
                    ctx.report_warning("attr-value", &val, Span::new(0, 0));
                }
                // Test for missing attribute
                if get_jsx_attr_string_value(&elem.attributes, "nonexistent", ctx).is_none() {
                    ctx.report_warning("attr-value", "none-for-missing", Span::new(0, 0));
                }
            }
        }
    }

    /// Rule that checks `get_jsx_attr_value`.
    #[derive(Debug)]
    struct AttrValueIdChecker;

    impl LintRule for AttrValueIdChecker {
        fn meta(&self) -> RuleMeta {
            RuleMeta {
                name: "attr-value-id".to_owned(),
                description: String::new(),
                category: Category::Correctness,
                default_severity: Severity::Warning,
            }
        }

        fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
            Some(&[AstNodeType::JSXOpeningElement])
        }

        fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
            if let AstNode::JSXOpeningElement(elem) = node {
                if let Some(value_id) = get_jsx_attr_value(&elem.attributes, "href", ctx) {
                    // Use get_string_value to extract the value
                    if let Some(val) = get_string_value(ctx, Some(value_id)) {
                        ctx.report_warning("attr-value-id", &val, Span::new(0, 0));
                    }
                }
                if get_jsx_attr_value(&elem.attributes, "nonexistent", ctx).is_none() {
                    ctx.report_warning("attr-value-id", "none-for-missing", Span::new(0, 0));
                }
            }
        }
    }

    #[test]
    fn test_has_jsx_attribute_found() {
        let source = r#"const el = <a href="https://example.com">link</a>;"#;
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(HasAttrChecker)];
        let diags = run_rules(source, "test.jsx", &rules);
        assert!(
            diags.iter().any(|d| d.message == "has-href"),
            "should find href attribute, got: {diags:?}"
        );
    }

    #[test]
    fn test_has_jsx_attribute_not_found() {
        let source = r#"const el = <a href="https://example.com">link</a>;"#;
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(HasAttrChecker)];
        let diags = run_rules(source, "test.jsx", &rules);
        assert!(
            diags.iter().any(|d| d.message == "no-missing"),
            "should not find 'missing' attribute"
        );
    }

    #[test]
    fn test_has_jsx_attribute_multiple() {
        let source = r#"const el = <a href="url" target="_blank">link</a>;"#;
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(HasAttrChecker)];
        let diags = run_rules(source, "test.jsx", &rules);
        assert!(
            diags.iter().any(|d| d.message == "has-href"),
            "should find href"
        );
        assert!(
            diags.iter().any(|d| d.message == "has-target"),
            "should find target"
        );
    }

    #[test]
    fn test_get_jsx_attr_string_value_found() {
        let source = r#"const el = <a href="https://example.com">link</a>;"#;
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(AttrStringValueChecker)];
        let diags = run_rules(source, "test.jsx", &rules);
        assert!(
            diags.iter().any(|d| d.message == "https://example.com"),
            "should extract href string value, got: {diags:?}"
        );
    }

    #[test]
    fn test_get_jsx_attr_string_value_missing() {
        let source = r#"const el = <a href="url">link</a>;"#;
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(AttrStringValueChecker)];
        let diags = run_rules(source, "test.jsx", &rules);
        assert!(
            diags.iter().any(|d| d.message == "none-for-missing"),
            "should return None for missing attribute"
        );
    }

    #[test]
    fn test_get_jsx_attr_value_and_get_string_value() {
        let source = r#"const el = <a href="https://example.com">link</a>;"#;
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(AttrValueIdChecker)];
        let diags = run_rules(source, "test.jsx", &rules);
        assert!(
            diags.iter().any(|d| d.message == "https://example.com"),
            "should extract href value via get_jsx_attr_value + get_string_value, got: {diags:?}"
        );
    }

    #[test]
    fn test_get_jsx_attr_value_missing() {
        let source = r#"const el = <a href="url">link</a>;"#;
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(AttrValueIdChecker)];
        let diags = run_rules(source, "test.jsx", &rules);
        assert!(
            diags.iter().any(|d| d.message == "none-for-missing"),
            "should return None for missing attribute"
        );
    }

    #[test]
    fn test_get_string_value_none_input() {
        let tree = starlint_ast::tree::AstTree::new();
        let ctx = LintContext::new(&tree, "", Path::new("test.jsx"));
        assert!(
            get_string_value(&ctx, None).is_none(),
            "None input should return None"
        );
    }

    #[test]
    fn test_get_string_value_invalid_node_id() {
        let tree = starlint_ast::tree::AstTree::new();
        let ctx = LintContext::new(&tree, "", Path::new("test.jsx"));
        assert!(
            get_string_value(&ctx, Some(NodeId(9999))).is_none(),
            "invalid NodeId should return None"
        );
    }

    #[test]
    fn test_has_jsx_attribute_empty_attributes() {
        let source = r"const el = <br />;";
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(HasAttrChecker)];
        let diags = run_rules(source, "test.jsx", &rules);
        assert!(
            diags.iter().any(|d| d.message == "no-missing"),
            "element with no attributes should not find 'missing'"
        );
        assert!(
            diags.iter().all(|d| d.message != "has-href"),
            "element with no attributes should not find 'href'"
        );
    }
}
