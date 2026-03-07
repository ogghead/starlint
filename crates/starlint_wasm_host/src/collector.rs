//! AST node collector for WASM plugins.
//!
//! [`NodeCollector`] traverses an [`AstTree`] and collects nodes that match
//! a plugin's declared [`NodeInterest`] flags, converting them to the
//! simplified [`WitAstNode`] representation for the WASM boundary.

use starlint_ast::node::AstNode;
use starlint_ast::tree::AstTree;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::Span;

use crate::bridge::{
    ArrayExpressionNode, ArrowFunctionExpressionNode, CallExpressionNode, DebuggerStatementNode,
    ExportDefaultNode, ExportNamedNode, FunctionDeclarationNode, IdentifierReferenceNode,
    ImportDeclarationNode, ImportSpecifierNode, JsxAttributeNode, JsxOpeningElementNode,
    MemberExpressionNode, NodeInterest, ObjectExpressionNode, StringLiteralNode,
    VariableDeclarationNode, VariableDeclaratorNode, WitAstNode,
};

/// Collects matching AST nodes during a single-pass traversal of [`AstTree`].
///
/// After calling [`collect`](Self::collect), the collected
/// nodes are available in [`into_nodes`](Self::into_nodes).
pub struct NodeCollector {
    /// Which node types to collect.
    interests: NodeInterest,
    /// Collected nodes.
    nodes: Vec<WitAstNode>,
}

impl NodeCollector {
    /// Create a new collector with the given interest flags.
    #[must_use]
    pub const fn new(interests: NodeInterest) -> Self {
        Self {
            interests,
            nodes: Vec::new(),
        }
    }

    /// Traverse the `AstTree` and collect matching nodes.
    #[allow(clippy::cast_possible_truncation, clippy::as_conversions)]
    pub fn collect(&mut self, tree: &AstTree) {
        for id in 0..tree.len() {
            let node_id = NodeId(id as u32);
            let Some(node) = tree.get(node_id) else {
                continue;
            };

            // Check if parent is an AwaitExpression (for call expression `is_awaited` flag).
            let is_awaited = tree
                .parent(node_id)
                .and_then(|pid| tree.get(pid))
                .is_some_and(|p| matches!(p, AstNode::AwaitExpression(_)));

            self.collect_node(tree, node_id, node, is_awaited);
        }
    }

    /// Consume the collector and return the collected nodes.
    #[must_use]
    pub fn into_nodes(self) -> Vec<WitAstNode> {
        self.nodes
    }

    /// Collect a single node if it matches an interest.
    #[allow(clippy::too_many_lines)]
    fn collect_node(&mut self, tree: &AstTree, _id: NodeId, node: &AstNode, is_awaited: bool) {
        match node {
            AstNode::ImportDeclaration(decl) if self.interests.import_declaration => {
                let specifiers = decl
                    .specifiers
                    .iter()
                    .filter_map(|spec_id| {
                        tree.get(*spec_id).and_then(|n| {
                            n.as_import_specifier().map(|spec| {
                                let imported = if spec.imported == "default" {
                                    None
                                } else if spec.imported == "*" {
                                    Some("*".to_owned())
                                } else {
                                    Some(spec.imported.clone())
                                };
                                ImportSpecifierNode {
                                    local: spec.local.clone(),
                                    imported,
                                }
                            })
                        })
                    })
                    .collect();

                self.nodes
                    .push(WitAstNode::ImportDecl(ImportDeclarationNode {
                        span: Span::new(decl.span.start, decl.span.end),
                        source: decl.source.clone(),
                        specifiers,
                    }));
            }

            AstNode::ExportDefaultDeclaration(decl)
                if self.interests.export_default_declaration =>
            {
                self.nodes
                    .push(WitAstNode::ExportDefaultDecl(ExportDefaultNode {
                        span: Span::new(decl.span.start, decl.span.end),
                    }));
            }

            AstNode::ExportNamedDeclaration(decl) if self.interests.export_named_declaration => {
                let names = decl
                    .specifiers
                    .iter()
                    .filter_map(|spec_id| {
                        tree.get(*spec_id)
                            .and_then(|n| n.as_export_specifier())
                            .map(|spec| spec.exported.clone())
                    })
                    .collect();

                self.nodes
                    .push(WitAstNode::ExportNamedDecl(ExportNamedNode {
                        span: Span::new(decl.span.start, decl.span.end),
                        names,
                    }));
            }

            AstNode::CallExpression(call) if self.interests.call_expression => {
                let callee_path = extract_callee_path_from_tree(tree, call.callee);
                let argument_count = u32::try_from(call.arguments.len()).unwrap_or(u32::MAX);

                self.nodes.push(WitAstNode::CallExpr(CallExpressionNode {
                    span: Span::new(call.span.start, call.span.end),
                    callee_path,
                    argument_count,
                    is_awaited,
                }));
            }

            AstNode::DebuggerStatement(stmt) if self.interests.debugger_statement => {
                self.nodes
                    .push(WitAstNode::DebuggerStmt(DebuggerStatementNode {
                        span: Span::new(stmt.span.start, stmt.span.end),
                    }));
            }

            AstNode::StaticMemberExpression(member) if self.interests.member_expression => {
                let object = extract_callee_path_from_tree(tree, member.object);
                self.nodes
                    .push(WitAstNode::MemberExpr(MemberExpressionNode {
                        span: Span::new(member.span.start, member.span.end),
                        object,
                        property: member.property.clone(),
                        computed: false,
                    }));
            }

            AstNode::ComputedMemberExpression(member) if self.interests.member_expression => {
                let object = extract_callee_path_from_tree(tree, member.object);
                self.nodes
                    .push(WitAstNode::MemberExpr(MemberExpressionNode {
                        span: Span::new(member.span.start, member.span.end),
                        object,
                        property: "<computed>".to_owned(),
                        computed: true,
                    }));
            }

            AstNode::IdentifierReference(id) if self.interests.identifier_reference => {
                self.nodes
                    .push(WitAstNode::IdentifierRef(IdentifierReferenceNode {
                        span: Span::new(id.span.start, id.span.end),
                        name: id.name.clone(),
                    }));
            }

            AstNode::ArrowFunctionExpression(arrow) if self.interests.arrow_function_expression => {
                // The custom parser may store `(a, b)` as a single SequenceExpression
                // param. Decompose it for an accurate count.
                let params_count = count_arrow_params(tree, &arrow.params);
                self.nodes
                    .push(WitAstNode::ArrowFnExpr(ArrowFunctionExpressionNode {
                        span: Span::new(arrow.span.start, arrow.span.end),
                        params_count,
                        is_async: arrow.is_async,
                        is_expression: arrow.expression,
                    }));
            }

            AstNode::Function(func) if self.interests.function_declaration => {
                let name = func.id.and_then(|id| {
                    tree.get(id)
                        .and_then(|n| n.as_binding_identifier())
                        .map(|bi| bi.name.clone())
                });
                let params_count = u32::try_from(func.params.len()).unwrap_or(u32::MAX);
                self.nodes.push(WitAstNode::FnDecl(FunctionDeclarationNode {
                    span: Span::new(func.span.start, func.span.end),
                    name,
                    params_count,
                    is_async: func.is_async,
                    is_generator: func.is_generator,
                }));
            }

            AstNode::VariableDeclaration(decl) if self.interests.variable_declaration => {
                let declarations = decl
                    .declarations
                    .iter()
                    .map(|d_id| {
                        let (binding_name, has_init) =
                            match tree.get(*d_id).and_then(|n| n.as_variable_declarator()) {
                                Some(d) => {
                                    let name = tree
                                        .get(d.id)
                                        .and_then(|n| n.as_binding_identifier())
                                        .map_or_else(
                                            || String::from("<pattern>"),
                                            |bi| bi.name.clone(),
                                        );
                                    (name, d.init.is_some())
                                }
                                None => (String::from("<unknown>"), false),
                            };
                        VariableDeclaratorNode {
                            name: binding_name,
                            has_init,
                        }
                    })
                    .collect();
                let decl_kind = decl.kind.as_str().to_owned();
                self.nodes
                    .push(WitAstNode::VarDecl(VariableDeclarationNode {
                        span: Span::new(decl.span.start, decl.span.end),
                        kind: decl_kind,
                        declarations,
                    }));
            }

            AstNode::StringLiteral(lit) if self.interests.string_literal => {
                self.nodes.push(WitAstNode::StringLit(StringLiteralNode {
                    span: Span::new(lit.span.start, lit.span.end),
                    value: lit.value.clone(),
                }));
            }

            AstNode::ObjectExpression(obj) if self.interests.object_expression => {
                let property_count = u32::try_from(obj.properties.len()).unwrap_or(u32::MAX);
                self.nodes
                    .push(WitAstNode::ObjectExpr(ObjectExpressionNode {
                        span: Span::new(obj.span.start, obj.span.end),
                        property_count,
                    }));
            }

            AstNode::ArrayExpression(arr) if self.interests.array_expression => {
                let element_count = u32::try_from(arr.elements.len()).unwrap_or(u32::MAX);
                self.nodes.push(WitAstNode::ArrayExpr(ArrayExpressionNode {
                    span: Span::new(arr.span.start, arr.span.end),
                    element_count,
                }));
            }

            AstNode::JSXElement(element) if self.interests.jsx_opening_element => {
                // Look up the opening element to get name, attributes, self-closing.
                let Some(opening) = tree
                    .get(element.opening_element)
                    .and_then(|n| n.as_jsx_opening_element())
                else {
                    return;
                };

                let name = opening.name.clone();

                let attributes = opening
                    .attributes
                    .iter()
                    .filter_map(|attr_id| {
                        let attr_node = tree.get(*attr_id)?;
                        match attr_node {
                            AstNode::JSXAttribute(attr) => {
                                let value = attr.value.and_then(|val_id| {
                                    tree.get(val_id)
                                        .and_then(|v| v.as_string_literal())
                                        .map(|s| s.value.clone())
                                });
                                Some(JsxAttributeNode {
                                    name: attr.name.clone(),
                                    value,
                                    is_spread: false,
                                })
                            }
                            AstNode::JSXSpreadAttribute(_) => Some(JsxAttributeNode {
                                name: "<spread>".to_owned(),
                                value: None,
                                is_spread: true,
                            }),
                            _ => None,
                        }
                    })
                    .collect();

                let children_count = u32::try_from(element.children.len()).unwrap_or(u32::MAX);

                self.nodes
                    .push(WitAstNode::JsxElement(JsxOpeningElementNode {
                        span: Span::new(opening.span.start, opening.span.end),
                        name,
                        attributes,
                        self_closing: opening.self_closing,
                        children_count,
                    }));
            }

            _ => {}
        }
    }
}

/// Count arrow function parameters, decomposing `SequenceExpression` params.
///
/// The custom parser may store `(a, b) => ...` with params = `[SequenceExpression([a, b])]`.
/// This function counts the actual parameter count by expanding sequence expressions.
fn count_arrow_params(tree: &AstTree, params: &[NodeId]) -> u32 {
    let mut count: u32 = 0;
    for pid in params {
        match tree.get(*pid) {
            Some(AstNode::SequenceExpression(seq)) => {
                count =
                    count.saturating_add(u32::try_from(seq.expressions.len()).unwrap_or(u32::MAX));
            }
            _ => {
                count = count.saturating_add(1);
            }
        }
    }
    count
}

/// Extract a dot-separated callee path from an `AstTree` node.
///
/// Returns `"<complex>"` for expressions that cannot be represented
/// as a simple dot path (computed member access, etc.).
fn extract_callee_path_from_tree(tree: &AstTree, id: NodeId) -> String {
    let Some(node) = tree.get(id) else {
        return "<complex>".to_owned();
    };
    match node {
        AstNode::IdentifierReference(ident) => ident.name.clone(),
        AstNode::StaticMemberExpression(member) => {
            let obj = extract_callee_path_from_tree(tree, member.object);
            format!("{obj}.{}", member.property)
        }
        _ => "<complex>".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    use std::path::Path;

    use starlint_parser::ParseOptions;

    /// Helper: parse source and return the `AstTree`.
    fn parse_tree(source: &str, file_path: &str) -> AstTree {
        let options = ParseOptions::from_path(Path::new(file_path));
        let result = starlint_parser::parse(source, options);
        result.tree
    }

    #[test]
    fn test_collect_debugger() {
        let tree = parse_tree("debugger;", "test.js");
        let interest = NodeInterest {
            debugger_statement: true,
            ..NodeInterest::default()
        };
        let mut collector = NodeCollector::new(interest);
        collector.collect(&tree);
        let nodes = collector.into_nodes();
        assert_eq!(nodes.len(), 1, "should collect one debugger statement");
        assert!(
            matches!(nodes.first(), Some(WitAstNode::DebuggerStmt(_))),
            "should be a DebuggerStmt"
        );
    }

    #[test]
    fn test_collect_import() {
        let tree = parse_tree("import { foo, bar } from 'my-module';", "test.js");
        let interest = NodeInterest {
            import_declaration: true,
            ..NodeInterest::default()
        };
        let mut collector = NodeCollector::new(interest);
        collector.collect(&tree);
        let nodes = collector.into_nodes();
        assert_eq!(nodes.len(), 1, "should collect one import");

        assert!(
            matches!(nodes.first(), Some(WitAstNode::ImportDecl(import)) if import.source == "my-module" && import.specifiers.len() == 2),
            "should be ImportDecl with source 'my-module' and 2 specifiers"
        );
    }

    #[test]
    fn test_collect_call_expression() {
        let tree = parse_tree("console.log('hello', 'world');", "test.js");
        let interest = NodeInterest {
            call_expression: true,
            ..NodeInterest::default()
        };
        let mut collector = NodeCollector::new(interest);
        collector.collect(&tree);
        let nodes = collector.into_nodes();
        assert_eq!(nodes.len(), 1, "should collect one call expression");

        assert!(
            matches!(nodes.first(), Some(WitAstNode::CallExpr(call)) if call.callee_path == "console.log" && call.argument_count == 2),
            "should be CallExpr with callee 'console.log' and 2 arguments"
        );
    }

    #[test]
    fn test_call_expression_awaited() {
        let tree = parse_tree(
            "async function f() { await fetch('url'); console.log(); }",
            "test.js",
        );
        let interest = NodeInterest {
            call_expression: true,
            ..NodeInterest::default()
        };
        let mut collector = NodeCollector::new(interest);
        collector.collect(&tree);
        let nodes = collector.into_nodes();
        assert_eq!(nodes.len(), 2, "should collect two call expressions");

        // First call: `fetch('url')` — awaited
        assert!(
            matches!(nodes.first(), Some(WitAstNode::CallExpr(call)) if call.is_awaited && call.callee_path == "fetch"),
            "fetch() should be marked as awaited"
        );
        // Second call: `console.log()` — not awaited
        assert!(
            matches!(nodes.get(1), Some(WitAstNode::CallExpr(call)) if !call.is_awaited && call.callee_path == "console.log"),
            "console.log() should not be marked as awaited"
        );
    }

    #[test]
    fn test_await_member_expression_call() {
        let tree = parse_tree("async function f() { await obj.method(); }", "test.js");
        let interest = NodeInterest {
            call_expression: true,
            ..NodeInterest::default()
        };
        let mut collector = NodeCollector::new(interest);
        collector.collect(&tree);
        let nodes = collector.into_nodes();
        assert_eq!(nodes.len(), 1, "should collect one call expression");
        assert!(
            matches!(nodes.first(), Some(WitAstNode::CallExpr(call)) if call.is_awaited && call.callee_path == "obj.method"),
            "await obj.method() should be marked as awaited"
        );
    }

    #[test]
    fn test_sibling_not_marked_awaited() {
        let tree = parse_tree(
            "async function f() { await fetch('url'); bar(); baz(); }",
            "test.js",
        );
        let interest = NodeInterest {
            call_expression: true,
            ..NodeInterest::default()
        };
        let mut collector = NodeCollector::new(interest);
        collector.collect(&tree);
        let nodes = collector.into_nodes();
        assert_eq!(nodes.len(), 3, "should collect three call expressions");
        // Only the first should be awaited.
        let awaited_count = nodes
            .iter()
            .filter(|n| matches!(n, WitAstNode::CallExpr(c) if c.is_awaited))
            .count();
        assert_eq!(
            awaited_count, 1,
            "only the awaited call should be marked, not siblings"
        );
    }

    #[test]
    fn test_collect_export_named() {
        let tree = parse_tree("const a = 1; const b = 2; export { a, b };", "test.js");
        let interest = NodeInterest {
            export_named_declaration: true,
            ..NodeInterest::default()
        };
        let mut collector = NodeCollector::new(interest);
        collector.collect(&tree);
        let nodes = collector.into_nodes();
        assert_eq!(nodes.len(), 1, "should collect one export");

        assert!(
            matches!(nodes.first(), Some(WitAstNode::ExportNamedDecl(export)) if export.names.len() == 2),
            "should be ExportNamedDecl with 2 names"
        );
    }

    #[test]
    fn test_no_collection_without_interest() {
        let tree = parse_tree("debugger; import 'foo'; console.log();", "test.js");
        let mut collector = NodeCollector::new(NodeInterest::default());
        collector.collect(&tree);
        let nodes = collector.into_nodes();
        assert!(nodes.is_empty(), "no interests should collect nothing");
    }

    #[test]
    fn test_collect_member_expression() {
        let tree = parse_tree("obj.prop;", "test.js");
        let interest = NodeInterest {
            member_expression: true,
            ..NodeInterest::default()
        };
        let mut collector = NodeCollector::new(interest);
        collector.collect(&tree);
        let nodes = collector.into_nodes();
        assert_eq!(nodes.len(), 1, "should collect one member expression");
        assert!(
            matches!(nodes.first(), Some(WitAstNode::MemberExpr(m)) if m.property == "prop" && !m.computed),
            "should be a static MemberExpr with property 'prop'"
        );
    }

    #[test]
    fn test_collect_identifier_reference() {
        let tree = parse_tree("foo;", "test.js");
        let interest = NodeInterest {
            identifier_reference: true,
            ..NodeInterest::default()
        };
        let mut collector = NodeCollector::new(interest);
        collector.collect(&tree);
        let nodes = collector.into_nodes();
        assert_eq!(nodes.len(), 1, "should collect one identifier reference");
        assert!(
            matches!(nodes.first(), Some(WitAstNode::IdentifierRef(id)) if id.name == "foo"),
            "should be IdentifierRef with name 'foo'"
        );
    }

    #[test]
    fn test_collect_arrow_function() {
        let tree = parse_tree("const f = (a, b) => a + b;", "test.js");
        let interest = NodeInterest {
            arrow_function_expression: true,
            ..NodeInterest::default()
        };
        let mut collector = NodeCollector::new(interest);
        collector.collect(&tree);
        let nodes = collector.into_nodes();
        assert_eq!(nodes.len(), 1, "should collect one arrow function");
        assert!(
            matches!(nodes.first(), Some(WitAstNode::ArrowFnExpr(arrow)) if arrow.params_count == 2 && arrow.is_expression),
            "should be ArrowFnExpr with 2 params and expression body"
        );
    }

    #[test]
    fn test_collect_function_declaration() {
        let tree = parse_tree("function greet(name) { return name; }", "test.js");
        let interest = NodeInterest {
            function_declaration: true,
            ..NodeInterest::default()
        };
        let mut collector = NodeCollector::new(interest);
        collector.collect(&tree);
        let nodes = collector.into_nodes();
        assert_eq!(nodes.len(), 1, "should collect one function declaration");
        assert!(
            matches!(nodes.first(), Some(WitAstNode::FnDecl(f)) if f.name.as_deref() == Some("greet") && f.params_count == 1),
            "should be FnDecl named 'greet' with 1 param"
        );
    }

    #[test]
    fn test_collect_variable_declaration() {
        let tree = parse_tree("const x = 1, y = 2;", "test.js");
        let interest = NodeInterest {
            variable_declaration: true,
            ..NodeInterest::default()
        };
        let mut collector = NodeCollector::new(interest);
        collector.collect(&tree);
        let nodes = collector.into_nodes();
        assert_eq!(nodes.len(), 1, "should collect one variable declaration");
        assert!(
            matches!(nodes.first(), Some(WitAstNode::VarDecl(v)) if v.declarations.len() == 2 && v.kind == "const"),
            "should be VarDecl with 2 declarators and kind 'const'"
        );
    }

    #[test]
    fn test_collect_string_literal() {
        let tree = parse_tree("'hello world';", "test.js");
        let interest = NodeInterest {
            string_literal: true,
            ..NodeInterest::default()
        };
        let mut collector = NodeCollector::new(interest);
        collector.collect(&tree);
        let nodes = collector.into_nodes();
        assert_eq!(nodes.len(), 1, "should collect one string literal");
        assert!(
            matches!(nodes.first(), Some(WitAstNode::StringLit(s)) if s.value == "hello world"),
            "should be StringLit with value 'hello world'"
        );
    }

    #[test]
    fn test_collect_object_expression() {
        let tree = parse_tree("({ a: 1, b: 2, c: 3 });", "test.js");
        let interest = NodeInterest {
            object_expression: true,
            ..NodeInterest::default()
        };
        let mut collector = NodeCollector::new(interest);
        collector.collect(&tree);
        let nodes = collector.into_nodes();
        assert_eq!(nodes.len(), 1, "should collect one object expression");
        assert!(
            matches!(nodes.first(), Some(WitAstNode::ObjectExpr(o)) if o.property_count == 3),
            "should be ObjectExpr with 3 properties"
        );
    }

    #[test]
    fn test_collect_array_expression() {
        let tree = parse_tree("[1, 2, 3, 4];", "test.js");
        let interest = NodeInterest {
            array_expression: true,
            ..NodeInterest::default()
        };
        let mut collector = NodeCollector::new(interest);
        collector.collect(&tree);
        let nodes = collector.into_nodes();
        assert_eq!(nodes.len(), 1, "should collect one array expression");
        assert!(
            matches!(nodes.first(), Some(WitAstNode::ArrayExpr(a)) if a.element_count == 4),
            "should be ArrayExpr with 4 elements"
        );
    }

    #[test]
    fn test_collect_jsx_self_closing() {
        let tree = parse_tree(
            r#"const el = <img src="photo.jpg" alt="A photo" />;"#,
            "test.jsx",
        );
        let interest = NodeInterest {
            jsx_opening_element: true,
            ..NodeInterest::default()
        };
        let mut collector = NodeCollector::new(interest);
        collector.collect(&tree);
        let nodes = collector.into_nodes();
        assert_eq!(nodes.len(), 1, "should collect one JSX element");
        assert!(
            matches!(
                nodes.first(),
                Some(WitAstNode::JsxElement(el))
                    if el.name == "img"
                        && el.attributes.len() == 2
                        && el.self_closing
                        && el.children_count == 0
            ),
            "should be a self-closing img with 2 attributes"
        );
    }

    #[test]
    fn test_collect_jsx_attribute_values() {
        let tree = parse_tree(
            r#"const el = <img src="photo.jpg" alt="A photo" />;"#,
            "test.jsx",
        );
        let interest = NodeInterest {
            jsx_opening_element: true,
            ..NodeInterest::default()
        };
        let mut collector = NodeCollector::new(interest);
        collector.collect(&tree);
        let nodes = collector.into_nodes();
        assert_eq!(nodes.len(), 1, "should collect one JSX element");

        assert!(
            matches!(
                nodes.first(),
                Some(WitAstNode::JsxElement(el))
                    if el.attributes.first().map(|a| a.name.as_str()) == Some("src")
                        && el.attributes.first().and_then(|a| a.value.as_deref()) == Some("photo.jpg")
            ),
            "first attribute should be src='photo.jpg'"
        );
    }

    #[test]
    fn test_collect_jsx_with_children() {
        let tree = parse_tree("const el = <div><span /><span /></div>;", "test.jsx");
        let interest = NodeInterest {
            jsx_opening_element: true,
            ..NodeInterest::default()
        };
        let mut collector = NodeCollector::new(interest);
        collector.collect(&tree);
        let nodes = collector.into_nodes();
        // Should collect: div (2 children), span (0), span (0)
        assert_eq!(nodes.len(), 3, "should collect three JSX elements");
        assert!(
            matches!(
                nodes.first(),
                Some(WitAstNode::JsxElement(div))
                    if div.name == "div"
                        && div.children_count == 2
                        && !div.self_closing
            ),
            "first element should be a non-self-closing div with 2 children"
        );
    }

    #[test]
    fn test_collect_jsx_spread_attribute() {
        let tree = parse_tree(
            r#"const el = <div {...props} className="test" />;"#,
            "test.jsx",
        );
        let interest = NodeInterest {
            jsx_opening_element: true,
            ..NodeInterest::default()
        };
        let mut collector = NodeCollector::new(interest);
        collector.collect(&tree);
        let nodes = collector.into_nodes();
        assert_eq!(nodes.len(), 1, "should collect one JSX element");
        assert!(
            matches!(
                nodes.first(),
                Some(WitAstNode::JsxElement(el))
                    if el.attributes.len() == 2
                        && el.attributes.first().is_some_and(|a| a.is_spread && a.name == "<spread>")
                        && el.attributes.get(1).is_some_and(|a| !a.is_spread && a.name == "className" && a.value.as_deref() == Some("test"))
            ),
            "should have spread + className attributes"
        );
    }

    #[test]
    fn test_collect_jsx_component_name() {
        let tree = parse_tree(r#"const el = <MyComponent foo="bar" />;"#, "test.jsx");
        let interest = NodeInterest {
            jsx_opening_element: true,
            ..NodeInterest::default()
        };
        let mut collector = NodeCollector::new(interest);
        collector.collect(&tree);
        let nodes = collector.into_nodes();
        assert_eq!(nodes.len(), 1, "should collect one JSX element");
        assert!(
            matches!(
                nodes.first(),
                Some(WitAstNode::JsxElement(el)) if el.name == "MyComponent"
            ),
            "should be JsxElement with name 'MyComponent'"
        );
    }

    #[test]
    fn test_no_jsx_collection_without_interest() {
        let tree = parse_tree("const el = <div />;", "test.jsx");
        let mut collector = NodeCollector::new(NodeInterest::default());
        collector.collect(&tree);
        let nodes = collector.into_nodes();
        assert!(nodes.is_empty(), "no interests should collect nothing");
    }
}
