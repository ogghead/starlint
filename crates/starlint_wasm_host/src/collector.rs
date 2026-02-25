//! AST node collector for WASM plugins.
//!
//! [`NodeCollector`] traverses an oxc AST and collects nodes that match
//! a plugin's declared [`NodeInterest`] flags, converting them to the
//! simplified [`WitAstNode`] representation for the WASM boundary.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast_visit::Visit;

use starlint_plugin_sdk::diagnostic::Span;

use crate::bridge::{
    ArrayExpressionNode, ArrowFunctionExpressionNode, CallExpressionNode, DebuggerStatementNode,
    ExportDefaultNode, ExportNamedNode, FunctionDeclarationNode, IdentifierReferenceNode,
    ImportDeclarationNode, ImportSpecifierNode, MemberExpressionNode, NodeInterest,
    ObjectExpressionNode, StringLiteralNode, VariableDeclarationNode, VariableDeclaratorNode,
    WitAstNode,
};

/// Collects matching AST nodes during a single-pass traversal.
///
/// After calling [`visit_program`](Visit::visit_program), the collected
/// nodes are available in [`into_nodes`](Self::into_nodes).
pub struct NodeCollector {
    /// Which node types to collect.
    interests: NodeInterest,
    /// Collected nodes.
    nodes: Vec<WitAstNode>,
    /// Whether the very next entered node is the direct child of an `AwaitExpression`.
    ///
    /// Set to `true` on `enter_node(AwaitExpression)`, then consumed/reset
    /// on the next `enter_node` call regardless of node type.
    await_direct_child: bool,
}

impl NodeCollector {
    /// Create a new collector with the given interest flags.
    #[must_use]
    pub const fn new(interests: NodeInterest) -> Self {
        Self {
            interests,
            nodes: Vec::new(),
            await_direct_child: false,
        }
    }

    /// Consume the collector and return the collected nodes.
    #[must_use]
    pub fn into_nodes(self) -> Vec<WitAstNode> {
        self.nodes
    }
}

impl<'a> Visit<'a> for NodeCollector {
    #[allow(clippy::too_many_lines)]
    fn enter_node(&mut self, kind: AstKind<'a>) {
        // Track whether this node is the direct child of an AwaitExpression.
        // After entering AwaitExpression, the very next enter_node is its argument.
        let is_awaited = self.await_direct_child;
        self.await_direct_child = matches!(kind, AstKind::AwaitExpression(_));

        match kind {
            AstKind::ImportDeclaration(decl) if self.interests.import_declaration => {
                let source = decl.source.value.to_string();
                let specifiers = decl
                    .specifiers
                    .as_ref()
                    .map(|specs| {
                        specs
                            .iter()
                            .map(|spec| {
                                match spec {
                                oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(s) => {
                                    ImportSpecifierNode {
                                        local: s.local.name.to_string(),
                                        imported: Some(s.imported.to_string()),
                                    }
                                }
                                oxc_ast::ast::ImportDeclarationSpecifier::ImportDefaultSpecifier(
                                    s,
                                ) => ImportSpecifierNode {
                                    local: s.local.name.to_string(),
                                    imported: None,
                                },
                                oxc_ast::ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(
                                    s,
                                ) => ImportSpecifierNode {
                                    local: s.local.name.to_string(),
                                    imported: Some("*".to_owned()),
                                },
                            }
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                self.nodes
                    .push(WitAstNode::ImportDecl(ImportDeclarationNode {
                        span: Span::new(decl.span.start, decl.span.end),
                        source,
                        specifiers,
                    }));
            }

            AstKind::ExportDefaultDeclaration(decl)
                if self.interests.export_default_declaration =>
            {
                self.nodes
                    .push(WitAstNode::ExportDefaultDecl(ExportDefaultNode {
                        span: Span::new(decl.span.start, decl.span.end),
                    }));
            }

            AstKind::ExportNamedDeclaration(decl) if self.interests.export_named_declaration => {
                let names = decl
                    .specifiers
                    .iter()
                    .map(|spec| spec.exported.to_string())
                    .collect();

                self.nodes
                    .push(WitAstNode::ExportNamedDecl(ExportNamedNode {
                        span: Span::new(decl.span.start, decl.span.end),
                        names,
                    }));
            }

            AstKind::CallExpression(call) if self.interests.call_expression => {
                let callee_path = extract_callee_path(&call.callee);
                let argument_count = u32::try_from(call.arguments.len()).unwrap_or(u32::MAX);

                self.nodes.push(WitAstNode::CallExpr(CallExpressionNode {
                    span: Span::new(call.span.start, call.span.end),
                    callee_path,
                    argument_count,
                    is_awaited,
                }));
            }

            AstKind::DebuggerStatement(stmt) if self.interests.debugger_statement => {
                self.nodes
                    .push(WitAstNode::DebuggerStmt(DebuggerStatementNode {
                        span: Span::new(stmt.span.start, stmt.span.end),
                    }));
            }

            AstKind::StaticMemberExpression(member) if self.interests.member_expression => {
                let object = extract_callee_path(&member.object);
                self.nodes
                    .push(WitAstNode::MemberExpr(MemberExpressionNode {
                        span: Span::new(member.span.start, member.span.end),
                        object,
                        property: member.property.name.to_string(),
                        computed: false,
                    }));
            }

            AstKind::ComputedMemberExpression(member) if self.interests.member_expression => {
                let object = extract_callee_path(&member.object);
                self.nodes
                    .push(WitAstNode::MemberExpr(MemberExpressionNode {
                        span: Span::new(member.span.start, member.span.end),
                        object,
                        property: "<computed>".to_owned(),
                        computed: true,
                    }));
            }

            AstKind::IdentifierReference(id) if self.interests.identifier_reference => {
                self.nodes
                    .push(WitAstNode::IdentifierRef(IdentifierReferenceNode {
                        span: Span::new(id.span.start, id.span.end),
                        name: id.name.to_string(),
                    }));
            }

            AstKind::ArrowFunctionExpression(arrow) if self.interests.arrow_function_expression => {
                let params_count = u32::try_from(arrow.params.items.len()).unwrap_or(u32::MAX);
                self.nodes
                    .push(WitAstNode::ArrowFnExpr(ArrowFunctionExpressionNode {
                        span: Span::new(arrow.span.start, arrow.span.end),
                        params_count,
                        is_async: arrow.r#async,
                        is_expression: arrow.expression,
                    }));
            }

            AstKind::Function(func) if self.interests.function_declaration => {
                let name = func.id.as_ref().map(|id| id.name.to_string());
                let params_count = u32::try_from(func.params.items.len()).unwrap_or(u32::MAX);
                self.nodes.push(WitAstNode::FnDecl(FunctionDeclarationNode {
                    span: Span::new(func.span.start, func.span.end),
                    name,
                    params_count,
                    is_async: func.r#async,
                    is_generator: func.generator,
                }));
            }

            AstKind::VariableDeclaration(decl) if self.interests.variable_declaration => {
                let declarations = decl
                    .declarations
                    .iter()
                    .map(|d| {
                        let binding_name =
                            d.id.get_identifier_name()
                                .as_deref()
                                .map_or_else(|| "<pattern>".to_owned(), ToString::to_string);
                        VariableDeclaratorNode {
                            name: binding_name,
                            has_init: d.init.is_some(),
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

            AstKind::StringLiteral(lit) if self.interests.string_literal => {
                self.nodes.push(WitAstNode::StringLit(StringLiteralNode {
                    span: Span::new(lit.span.start, lit.span.end),
                    value: lit.value.to_string(),
                }));
            }

            AstKind::ObjectExpression(obj) if self.interests.object_expression => {
                let property_count = u32::try_from(obj.properties.len()).unwrap_or(u32::MAX);
                self.nodes
                    .push(WitAstNode::ObjectExpr(ObjectExpressionNode {
                        span: Span::new(obj.span.start, obj.span.end),
                        property_count,
                    }));
            }

            AstKind::ArrayExpression(arr) if self.interests.array_expression => {
                let element_count = u32::try_from(arr.elements.len()).unwrap_or(u32::MAX);
                self.nodes.push(WitAstNode::ArrayExpr(ArrayExpressionNode {
                    span: Span::new(arr.span.start, arr.span.end),
                    element_count,
                }));
            }

            _ => {}
        }
    }
}

/// Extract a dot-separated callee path from an expression.
///
/// Returns `"<complex>"` for expressions that cannot be represented
/// as a simple dot path (computed member access, etc.).
fn extract_callee_path(expr: &Expression<'_>) -> String {
    match expr {
        Expression::Identifier(id) => id.name.to_string(),
        Expression::StaticMemberExpression(member) => {
            let obj = extract_callee_path(&member.object);
            format!("{obj}.{}", member.property.name)
        }
        _ => "<complex>".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    use std::path::Path;

    use oxc_allocator::Allocator;

    use starlint_core::parser::parse_file;

    #[test]
    fn test_collect_debugger() {
        let allocator = Allocator::default();
        let source = "debugger;";
        let parsed = parse_file(&allocator, source, Path::new("test.js"));
        assert!(parsed.is_ok(), "parse should succeed");

        if let Ok(result) = parsed {
            let interest = NodeInterest {
                debugger_statement: true,
                ..NodeInterest::default()
            };
            let mut collector = NodeCollector::new(interest);
            collector.visit_program(&result.program);
            let nodes = collector.into_nodes();
            assert_eq!(nodes.len(), 1, "should collect one debugger statement");
            let first = nodes.first();
            assert!(
                matches!(first, Some(WitAstNode::DebuggerStmt(_))),
                "should be a DebuggerStmt"
            );
        }
    }

    #[test]
    fn test_collect_import() {
        let allocator = Allocator::default();
        let source = "import { foo, bar } from 'my-module';";
        let parsed = parse_file(&allocator, source, Path::new("test.js"));
        assert!(parsed.is_ok(), "parse should succeed");

        if let Ok(result) = parsed {
            let interest = NodeInterest {
                import_declaration: true,
                ..NodeInterest::default()
            };
            let mut collector = NodeCollector::new(interest);
            collector.visit_program(&result.program);
            let nodes = collector.into_nodes();
            assert_eq!(nodes.len(), 1, "should collect one import");

            let first = nodes.first();
            assert!(
                matches!(first, Some(WitAstNode::ImportDecl(import)) if import.source == "my-module" && import.specifiers.len() == 2),
                "should be ImportDecl with source 'my-module' and 2 specifiers"
            );
        }
    }

    #[test]
    fn test_collect_call_expression() {
        let allocator = Allocator::default();
        let source = "console.log('hello', 'world');";
        let parsed = parse_file(&allocator, source, Path::new("test.js"));
        assert!(parsed.is_ok(), "parse should succeed");

        if let Ok(result) = parsed {
            let interest = NodeInterest {
                call_expression: true,
                ..NodeInterest::default()
            };
            let mut collector = NodeCollector::new(interest);
            collector.visit_program(&result.program);
            let nodes = collector.into_nodes();
            assert_eq!(nodes.len(), 1, "should collect one call expression");

            let first = nodes.first();
            assert!(
                matches!(first, Some(WitAstNode::CallExpr(call)) if call.callee_path == "console.log" && call.argument_count == 2),
                "should be CallExpr with callee 'console.log' and 2 arguments"
            );
        }
    }

    #[test]
    fn test_call_expression_awaited() {
        let allocator = Allocator::default();
        let source = "async function f() { await fetch('url'); console.log(); }";
        let parsed = parse_file(&allocator, source, Path::new("test.js"));
        assert!(parsed.is_ok(), "parse should succeed");

        if let Ok(result) = parsed {
            let interest = NodeInterest {
                call_expression: true,
                ..NodeInterest::default()
            };
            let mut collector = NodeCollector::new(interest);
            collector.visit_program(&result.program);
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
    }

    #[test]
    fn test_collect_export_named() {
        let allocator = Allocator::default();
        let source = "const a = 1; const b = 2; export { a, b };";
        let parsed = parse_file(&allocator, source, Path::new("test.js"));
        assert!(parsed.is_ok(), "parse should succeed");

        if let Ok(result) = parsed {
            let interest = NodeInterest {
                export_named_declaration: true,
                ..NodeInterest::default()
            };
            let mut collector = NodeCollector::new(interest);
            collector.visit_program(&result.program);
            let nodes = collector.into_nodes();
            assert_eq!(nodes.len(), 1, "should collect one export");

            let first = nodes.first();
            assert!(
                matches!(first, Some(WitAstNode::ExportNamedDecl(export)) if export.names.len() == 2),
                "should be ExportNamedDecl with 2 names"
            );
        }
    }

    #[test]
    fn test_no_collection_without_interest() {
        let allocator = Allocator::default();
        let source = "debugger; import 'foo'; console.log();";
        let parsed = parse_file(&allocator, source, Path::new("test.js"));
        assert!(parsed.is_ok(), "parse should succeed");

        if let Ok(result) = parsed {
            let mut collector = NodeCollector::new(NodeInterest::default());
            collector.visit_program(&result.program);
            let nodes = collector.into_nodes();
            assert!(nodes.is_empty(), "no interests should collect nothing");
        }
    }

    #[test]
    fn test_collect_member_expression() {
        let allocator = Allocator::default();
        let source = "obj.prop;";
        let parsed = parse_file(&allocator, source, Path::new("test.js"));
        assert!(parsed.is_ok(), "parse should succeed");

        if let Ok(result) = parsed {
            let interest = NodeInterest {
                member_expression: true,
                ..NodeInterest::default()
            };
            let mut collector = NodeCollector::new(interest);
            collector.visit_program(&result.program);
            let nodes = collector.into_nodes();
            assert_eq!(nodes.len(), 1, "should collect one member expression");
            assert!(
                matches!(nodes.first(), Some(WitAstNode::MemberExpr(m)) if m.property == "prop" && !m.computed),
                "should be a static MemberExpr with property 'prop'"
            );
        }
    }

    #[test]
    fn test_collect_identifier_reference() {
        let allocator = Allocator::default();
        let source = "foo;";
        let parsed = parse_file(&allocator, source, Path::new("test.js"));
        assert!(parsed.is_ok(), "parse should succeed");

        if let Ok(result) = parsed {
            let interest = NodeInterest {
                identifier_reference: true,
                ..NodeInterest::default()
            };
            let mut collector = NodeCollector::new(interest);
            collector.visit_program(&result.program);
            let nodes = collector.into_nodes();
            assert_eq!(nodes.len(), 1, "should collect one identifier reference");
            assert!(
                matches!(nodes.first(), Some(WitAstNode::IdentifierRef(id)) if id.name == "foo"),
                "should be IdentifierRef with name 'foo'"
            );
        }
    }

    #[test]
    fn test_collect_arrow_function() {
        let allocator = Allocator::default();
        let source = "const f = (a, b) => a + b;";
        let parsed = parse_file(&allocator, source, Path::new("test.js"));
        assert!(parsed.is_ok(), "parse should succeed");

        if let Ok(result) = parsed {
            let interest = NodeInterest {
                arrow_function_expression: true,
                ..NodeInterest::default()
            };
            let mut collector = NodeCollector::new(interest);
            collector.visit_program(&result.program);
            let nodes = collector.into_nodes();
            assert_eq!(nodes.len(), 1, "should collect one arrow function");
            assert!(
                matches!(nodes.first(), Some(WitAstNode::ArrowFnExpr(arrow)) if arrow.params_count == 2 && arrow.is_expression),
                "should be ArrowFnExpr with 2 params and expression body"
            );
        }
    }

    #[test]
    fn test_collect_function_declaration() {
        let allocator = Allocator::default();
        let source = "function greet(name) { return name; }";
        let parsed = parse_file(&allocator, source, Path::new("test.js"));
        assert!(parsed.is_ok(), "parse should succeed");

        if let Ok(result) = parsed {
            let interest = NodeInterest {
                function_declaration: true,
                ..NodeInterest::default()
            };
            let mut collector = NodeCollector::new(interest);
            collector.visit_program(&result.program);
            let nodes = collector.into_nodes();
            assert_eq!(nodes.len(), 1, "should collect one function declaration");
            assert!(
                matches!(nodes.first(), Some(WitAstNode::FnDecl(f)) if f.name.as_deref() == Some("greet") && f.params_count == 1),
                "should be FnDecl named 'greet' with 1 param"
            );
        }
    }

    #[test]
    fn test_collect_variable_declaration() {
        let allocator = Allocator::default();
        let source = "const x = 1, y = 2;";
        let parsed = parse_file(&allocator, source, Path::new("test.js"));
        assert!(parsed.is_ok(), "parse should succeed");

        if let Ok(result) = parsed {
            let interest = NodeInterest {
                variable_declaration: true,
                ..NodeInterest::default()
            };
            let mut collector = NodeCollector::new(interest);
            collector.visit_program(&result.program);
            let nodes = collector.into_nodes();
            assert_eq!(nodes.len(), 1, "should collect one variable declaration");
            assert!(
                matches!(nodes.first(), Some(WitAstNode::VarDecl(v)) if v.declarations.len() == 2 && v.kind == "const"),
                "should be VarDecl with 2 declarators and kind 'const'"
            );
        }
    }

    #[test]
    fn test_collect_string_literal() {
        let allocator = Allocator::default();
        let source = "'hello world';";
        let parsed = parse_file(&allocator, source, Path::new("test.js"));
        assert!(parsed.is_ok(), "parse should succeed");

        if let Ok(result) = parsed {
            let interest = NodeInterest {
                string_literal: true,
                ..NodeInterest::default()
            };
            let mut collector = NodeCollector::new(interest);
            collector.visit_program(&result.program);
            let nodes = collector.into_nodes();
            assert_eq!(nodes.len(), 1, "should collect one string literal");
            assert!(
                matches!(nodes.first(), Some(WitAstNode::StringLit(s)) if s.value == "hello world"),
                "should be StringLit with value 'hello world'"
            );
        }
    }

    #[test]
    fn test_collect_object_expression() {
        let allocator = Allocator::default();
        let source = "({ a: 1, b: 2, c: 3 });";
        let parsed = parse_file(&allocator, source, Path::new("test.js"));
        assert!(parsed.is_ok(), "parse should succeed");

        if let Ok(result) = parsed {
            let interest = NodeInterest {
                object_expression: true,
                ..NodeInterest::default()
            };
            let mut collector = NodeCollector::new(interest);
            collector.visit_program(&result.program);
            let nodes = collector.into_nodes();
            assert_eq!(nodes.len(), 1, "should collect one object expression");
            assert!(
                matches!(nodes.first(), Some(WitAstNode::ObjectExpr(o)) if o.property_count == 3),
                "should be ObjectExpr with 3 properties"
            );
        }
    }

    #[test]
    fn test_collect_array_expression() {
        let allocator = Allocator::default();
        let source = "[1, 2, 3, 4];";
        let parsed = parse_file(&allocator, source, Path::new("test.js"));
        assert!(parsed.is_ok(), "parse should succeed");

        if let Ok(result) = parsed {
            let interest = NodeInterest {
                array_expression: true,
                ..NodeInterest::default()
            };
            let mut collector = NodeCollector::new(interest);
            collector.visit_program(&result.program);
            let nodes = collector.into_nodes();
            assert_eq!(nodes.len(), 1, "should collect one array expression");
            assert!(
                matches!(nodes.first(), Some(WitAstNode::ArrayExpr(a)) if a.element_count == 4),
                "should be ArrayExpr with 4 elements"
            );
        }
    }
}
