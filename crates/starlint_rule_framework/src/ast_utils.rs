//! Shared AST utilities for lint rules.
//!
//! Provides helpers for extracting information from AST nodes,
//! such as property key names and literal detection.

use starlint_ast::node::AstNode;
use starlint_ast::types::NodeId;

use crate::LintContext;

/// Extract a static key name from a property/method key [`NodeId`].
///
/// Handles the common cases: identifier names, string literals, and
/// numeric literals. Returns `None` for computed keys or unsupported
/// node types.
///
/// # Supported node types
///
/// - [`IdentifierReference`](AstNode::IdentifierReference) → `ident.name`
/// - [`BindingIdentifier`](AstNode::BindingIdentifier) → `ident.name`
/// - [`StringLiteral`](AstNode::StringLiteral) → `lit.value`
/// - [`NumericLiteral`](AstNode::NumericLiteral) → `lit.raw`
#[must_use]
pub fn extract_static_key_name(key_id: NodeId, ctx: &LintContext<'_>) -> Option<String> {
    match ctx.node(key_id)? {
        AstNode::IdentifierReference(ident) => Some(ident.name.clone()),
        AstNode::BindingIdentifier(ident) => Some(ident.name.clone()),
        AstNode::StringLiteral(lit) => Some(lit.value.clone()),
        AstNode::NumericLiteral(lit) => Some(lit.raw.clone()),
        _ => None,
    }
}

/// Check if an expression is an `expect(...)` call or a chain like
/// `expect(...).not`.
///
/// Recursively walks member expression chains (e.g., `expect(x).not.to`)
/// to find the root `expect()` call.
#[must_use]
pub fn is_expect_chain(expr_id: NodeId, ctx: &LintContext<'_>) -> bool {
    match ctx.node(expr_id) {
        Some(AstNode::CallExpression(call)) => {
            matches!(ctx.node(call.callee), Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "expect")
        }
        Some(AstNode::StaticMemberExpression(member)) => is_expect_chain(member.object, ctx),
        _ => false,
    }
}

/// Walk up the AST parent chain to check if `node_id` is inside a
/// callback of a function matching one of `names`.
///
/// Returns `true` if an ancestor `CallExpression` has a callee that is an
/// `IdentifierReference` whose name is in `names`.
#[must_use]
pub fn is_inside_call_with_names(node_id: NodeId, ctx: &LintContext<'_>, names: &[&str]) -> bool {
    let tree = ctx.tree();
    let mut current = tree.parent(node_id);
    while let Some(pid) = current {
        if let Some(AstNode::CallExpression(call)) = tree.get(pid) {
            if let Some(AstNode::IdentifierReference(id)) = tree.get(call.callee) {
                if names.contains(&id.name.as_str()) {
                    return true;
                }
            }
        }
        current = tree.parent(pid);
    }
    false
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

    // ── Helper: collect diagnostics whose message contains findings ──

    /// Rule that tests `extract_static_key_name` on property keys.
    #[derive(Debug)]
    struct KeyNameCollector;

    impl LintRule for KeyNameCollector {
        fn meta(&self) -> RuleMeta {
            RuleMeta {
                name: "key-name-collector".to_owned(),
                description: String::new(),
                category: Category::Correctness,
                default_severity: Severity::Warning,
            }
        }

        fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
            Some(&[AstNodeType::PropertyDefinition])
        }

        fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
            if let AstNode::PropertyDefinition(prop) = node {
                if let Some(name) = extract_static_key_name(prop.key, ctx) {
                    ctx.report_warning("key-name-collector", &name, Span::new(0, 0));
                }
            }
        }
    }

    /// Rule that tests `is_expect_chain` on call expressions.
    #[derive(Debug)]
    struct ExpectChainChecker;

    impl LintRule for ExpectChainChecker {
        fn meta(&self) -> RuleMeta {
            RuleMeta {
                name: "expect-chain".to_owned(),
                description: String::new(),
                category: Category::Correctness,
                default_severity: Severity::Warning,
            }
        }

        fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
            Some(&[AstNodeType::CallExpression])
        }

        fn run(&self, node_id: NodeId, _node: &AstNode, ctx: &mut LintContext<'_>) {
            if is_expect_chain(node_id, ctx) {
                ctx.report_warning("expect-chain", "found-expect", Span::new(0, 0));
            }
        }
    }

    /// Rule that tests `is_inside_call_with_names` on identifier references.
    #[derive(Debug)]
    struct InsideCallChecker;

    impl LintRule for InsideCallChecker {
        fn meta(&self) -> RuleMeta {
            RuleMeta {
                name: "inside-call".to_owned(),
                description: String::new(),
                category: Category::Correctness,
                default_severity: Severity::Warning,
            }
        }

        fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
            Some(&[AstNodeType::NumericLiteral])
        }

        fn run(&self, node_id: NodeId, _node: &AstNode, ctx: &mut LintContext<'_>) {
            if is_inside_call_with_names(node_id, ctx, &["describe", "it"]) {
                ctx.report_warning("inside-call", "inside-test-fn", Span::new(0, 0));
            } else {
                ctx.report_warning("inside-call", "not-inside-test-fn", Span::new(0, 0));
            }
        }
    }

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

    #[test]
    fn test_extract_static_key_name_identifier() {
        // PropertyDefinition with an identifier key
        let source = "class Foo { myProp = 1; }";
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(KeyNameCollector)];
        let diags = run_rules(source, "test.js", &rules);
        assert!(
            diags.iter().any(|d| d.message == "myProp"),
            "should extract identifier key name 'myProp', got: {diags:?}"
        );
    }

    #[test]
    fn test_extract_static_key_name_string_literal() {
        let source = "class Foo { 'stringKey' = 1; }";
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(KeyNameCollector)];
        let diags = run_rules(source, "test.js", &rules);
        assert!(
            diags.iter().any(|d| d.message == "stringKey"),
            "should extract string literal key name, got: {diags:?}"
        );
    }

    #[test]
    fn test_extract_static_key_name_numeric_literal() {
        let source = "class Foo { 42 = 'val'; }";
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(KeyNameCollector)];
        let diags = run_rules(source, "test.js", &rules);
        assert!(
            diags.iter().any(|d| d.message == "42"),
            "should extract numeric literal key name, got: {diags:?}"
        );
    }

    #[test]
    fn test_extract_static_key_name_computed_identifier() {
        // Computed keys with identifier references still resolve to the identifier name
        // since `extract_static_key_name` looks at the key node type, not the `computed` flag.
        let source = "class Foo { [computed] = 1; }";
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(KeyNameCollector)];
        let diags = run_rules(source, "test.js", &rules);
        assert!(
            diags.iter().any(|d| d.message == "computed"),
            "computed identifier key should extract the identifier name, got: {diags:?}"
        );
    }

    #[test]
    fn test_is_expect_chain_direct_call() {
        let source = "expect(1);";
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ExpectChainChecker)];
        let diags = run_rules(source, "test.js", &rules);
        assert!(
            diags.iter().any(|d| d.message == "found-expect"),
            "expect(1) should be detected as an expect chain"
        );
    }

    #[test]
    fn test_is_expect_chain_member_expression() {
        let source = "expect(1).not;";
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ExpectChainChecker)];
        let diags = run_rules(source, "test.js", &rules);
        assert!(
            diags.iter().any(|d| d.message == "found-expect"),
            "expect(1).not should be detected as having an expect chain"
        );
    }

    #[test]
    fn test_is_expect_chain_non_expect() {
        let source = "foo(1);";
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ExpectChainChecker)];
        let diags = run_rules(source, "test.js", &rules);
        assert!(
            diags.iter().all(|d| d.message != "found-expect"),
            "foo(1) should not be detected as an expect chain"
        );
    }

    #[test]
    fn test_is_inside_call_with_names_true() {
        let source = "describe('test', function() { 42; });";
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(InsideCallChecker)];
        let diags = run_rules(source, "test.js", &rules);
        assert!(
            diags.iter().any(|d| d.message == "inside-test-fn"),
            "42 inside describe() should be detected, got: {diags:?}"
        );
    }

    #[test]
    fn test_is_inside_call_with_names_false() {
        let source = "42;";
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(InsideCallChecker)];
        let diags = run_rules(source, "test.js", &rules);
        assert!(
            diags.iter().any(|d| d.message == "not-inside-test-fn"),
            "42 at top level should not be inside test fn, got: {diags:?}"
        );
    }

    #[test]
    fn test_extract_static_key_name_invalid_node_id() {
        // Test with an invalid NodeId — should return None
        let tree = starlint_ast::tree::AstTree::new();
        let ctx = LintContext::new(&tree, "", Path::new("test.js"));
        let result = extract_static_key_name(NodeId(9999), &ctx);
        assert!(result.is_none(), "invalid NodeId should return None");
    }

    #[test]
    fn test_is_expect_chain_invalid_node_id() {
        let tree = starlint_ast::tree::AstTree::new();
        let ctx = LintContext::new(&tree, "", Path::new("test.js"));
        assert!(
            !is_expect_chain(NodeId(9999), &ctx),
            "invalid NodeId should return false"
        );
    }

    /// Rule that tests `is_inside_call_with_names` with an empty names list.
    #[derive(Debug)]
    struct EmptyNamesChecker;

    impl LintRule for EmptyNamesChecker {
        fn meta(&self) -> RuleMeta {
            RuleMeta {
                name: "empty-names".to_owned(),
                description: String::new(),
                category: Category::Correctness,
                default_severity: Severity::Warning,
            }
        }

        fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
            Some(&[AstNodeType::NumericLiteral])
        }

        fn run(&self, node_id: NodeId, _node: &AstNode, ctx: &mut LintContext<'_>) {
            if is_inside_call_with_names(node_id, ctx, &[]) {
                ctx.report_warning("empty-names", "found", Span::new(0, 0));
            } else {
                ctx.report_warning("empty-names", "not-found", Span::new(0, 0));
            }
        }
    }

    #[test]
    fn test_is_inside_call_with_names_empty_names() {
        let source = "describe('test', function() { 42; });";
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(EmptyNamesChecker)];
        let diags = run_rules(source, "test.js", &rules);
        assert!(
            diags.iter().any(|d| d.message == "not-found"),
            "empty names list should never match"
        );
    }
}
