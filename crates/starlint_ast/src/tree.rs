//! Flat indexed AST container.
//!
//! [`AstTree`] owns all nodes in a contiguous `Vec`, with parent pointers
//! stored in a parallel `Vec`. Child references within nodes use [`NodeId`]
//! indices.

use crate::node::AstNode;
use crate::node_type::AstNodeType;
use crate::types::{NodeId, Span};

/// A flat indexed AST. Nodes reference children by [`NodeId`] index.
#[derive(Debug, Clone)]
pub struct AstTree {
    /// All nodes in depth-first preorder.
    nodes: Vec<AstNode>,
    /// Parent of each node (parallel to `nodes`). `None` for root.
    parents: Vec<Option<NodeId>>,
}

impl AstTree {
    /// Create a new empty tree.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            nodes: Vec::new(),
            parents: Vec::new(),
        }
    }

    /// Create a tree with pre-allocated capacity.
    #[must_use]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(cap),
            parents: Vec::with_capacity(cap),
        }
    }

    /// Push a node, returning its [`NodeId`].
    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
    pub fn push(&mut self, node: AstNode, parent: Option<NodeId>) -> NodeId {
        let id = NodeId(self.nodes.len() as u32);
        self.nodes.push(node);
        self.parents.push(parent);
        id
    }

    /// Number of nodes in the tree.
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the tree is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Get a node by ID.
    ///
    /// Returns `None` if the ID is out of bounds.
    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<&AstNode> {
        self.nodes.get(id.index())
    }

    /// Get a node by ID.
    ///
    /// Returns `None` if the ID is out of bounds.
    #[must_use]
    pub fn node(&self, id: NodeId) -> Option<&AstNode> {
        self.nodes.get(id.index())
    }

    /// Get the parent of a node.
    #[must_use]
    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.parents.get(id.index()).copied().flatten()
    }

    /// Get the [`AstNodeType`] for a node.
    #[must_use]
    pub fn node_type(&self, id: NodeId) -> Option<AstNodeType> {
        self.get(id).map(AstNodeType::from)
    }

    /// Get the span of a node.
    #[must_use]
    pub fn span(&self, id: NodeId) -> Option<Span> {
        self.get(id).map(AstNode::span)
    }

    /// Iterate over all (`NodeId`, `&AstNode`) pairs in insertion order.
    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &AstNode)> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (NodeId(i as u32), node))
    }

    /// Get all ancestor node IDs from `id` up to the root (exclusive of `id`).
    #[must_use]
    pub fn ancestors(&self, id: NodeId) -> Vec<NodeId> {
        let mut result = Vec::new();
        let mut current = self.parent(id);
        while let Some(parent_id) = current {
            result.push(parent_id);
            current = self.parent(parent_id);
        }
        result
    }

    /// Collect child [`NodeId`]s for a given node by inspecting its fields.
    ///
    /// This is a convenience method — for high-performance traversal, use
    /// [`TreeCursor`](crate::TreeCursor) instead.
    #[must_use]
    pub fn children(&self, id: NodeId) -> Vec<NodeId> {
        let Some(node) = self.get(id) else {
            return Vec::new();
        };
        node_children(node)
    }

    /// Borrow the underlying nodes slice.
    #[must_use]
    pub fn nodes(&self) -> &[AstNode] {
        &self.nodes
    }

    /// Borrow the underlying parents slice.
    #[must_use]
    pub fn parents(&self) -> &[Option<NodeId>] {
        &self.parents
    }
}

impl Default for AstTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract child `NodeId`s from a node's fields.
#[allow(clippy::too_many_lines)]
fn node_children(node: &AstNode) -> Vec<NodeId> {
    match node {
        AstNode::Program(n) => n.body.to_vec(),
        AstNode::BlockStatement(n) => n.body.to_vec(),
        AstNode::IfStatement(n) => {
            let mut c = vec![n.test, n.consequent];
            if let Some(alt) = n.alternate {
                c.push(alt);
            }
            c
        }
        AstNode::SwitchStatement(n) => {
            let mut c = vec![n.discriminant];
            c.extend_from_slice(&n.cases);
            c
        }
        AstNode::SwitchCase(n) => {
            let mut c = Vec::new();
            if let Some(test) = n.test {
                c.push(test);
            }
            c.extend_from_slice(&n.consequent);
            c
        }
        AstNode::ForStatement(n) => {
            let mut c = Vec::new();
            if let Some(init) = n.init {
                c.push(init);
            }
            if let Some(test) = n.test {
                c.push(test);
            }
            if let Some(update) = n.update {
                c.push(update);
            }
            c.push(n.body);
            c
        }
        AstNode::ForInStatement(n) => vec![n.left, n.right, n.body],
        AstNode::ForOfStatement(n) => vec![n.left, n.right, n.body],
        AstNode::WhileStatement(n) => vec![n.test, n.body],
        AstNode::DoWhileStatement(n) => vec![n.body, n.test],
        AstNode::TryStatement(n) => {
            let mut c = vec![n.block];
            if let Some(h) = n.handler {
                c.push(h);
            }
            if let Some(f) = n.finalizer {
                c.push(f);
            }
            c
        }
        AstNode::CatchClause(n) => {
            let mut c = Vec::new();
            if let Some(p) = n.param {
                c.push(p);
            }
            c.push(n.body);
            c
        }
        AstNode::ThrowStatement(n) => vec![n.argument],
        AstNode::ReturnStatement(n) => n.argument.into_iter().collect(),
        AstNode::LabeledStatement(n) => vec![n.body],
        AstNode::WithStatement(n) => vec![n.object, n.body],
        AstNode::ExpressionStatement(n) => vec![n.expression],
        AstNode::VariableDeclaration(n) => n.declarations.to_vec(),
        AstNode::VariableDeclarator(n) => {
            let mut c = vec![n.id];
            if let Some(init) = n.init {
                c.push(init);
            }
            c
        }
        AstNode::Function(n) => {
            let mut c = Vec::new();
            if let Some(id) = n.id {
                c.push(id);
            }
            c.extend_from_slice(&n.params);
            if let Some(body) = n.body {
                c.push(body);
            }
            c
        }
        AstNode::FunctionBody(n) => n.statements.to_vec(),
        AstNode::Class(n) => {
            let mut c = Vec::new();
            if let Some(id) = n.id {
                c.push(id);
            }
            if let Some(sc) = n.super_class {
                c.push(sc);
            }
            c.extend_from_slice(&n.body);
            c
        }
        AstNode::StaticBlock(n) => n.body.to_vec(),
        AstNode::CallExpression(n) => {
            let mut c = vec![n.callee];
            c.extend_from_slice(&n.arguments);
            c
        }
        AstNode::NewExpression(n) => {
            let mut c = vec![n.callee];
            c.extend_from_slice(&n.arguments);
            c
        }
        AstNode::BinaryExpression(n) => vec![n.left, n.right],
        AstNode::LogicalExpression(n) => vec![n.left, n.right],
        AstNode::AssignmentExpression(n) => vec![n.left, n.right],
        AstNode::UnaryExpression(n) => vec![n.argument],
        AstNode::UpdateExpression(n) => vec![n.argument],
        AstNode::ConditionalExpression(n) => vec![n.test, n.consequent, n.alternate],
        AstNode::SequenceExpression(n) => n.expressions.to_vec(),
        AstNode::ContinueStatement(_)
        | AstNode::EmptyStatement(_)
        | AstNode::IdentifierReference(_)
        | AstNode::BindingIdentifier(_)
        | AstNode::StringLiteral(_)
        | AstNode::NumericLiteral(_)
        | AstNode::NullLiteral(_)
        | AstNode::RegExpLiteral(_)
        | AstNode::ThisExpression(_)
        | AstNode::DebuggerStatement(_)
        | AstNode::TSAnyKeyword(_)
        | AstNode::TSVoidKeyword(_)
        | AstNode::JSXText(_)
        | AstNode::ImportSpecifier(_)
        | AstNode::ExportSpecifier(_)
        | AstNode::ExportAllDeclaration(_)
        | AstNode::JSXNamespacedName(_) => Vec::new(),
        AstNode::TemplateLiteral(n) => n.expressions.to_vec(),
        AstNode::TaggedTemplateExpression(n) => vec![n.tag, n.quasi],
        AstNode::ArrayExpression(n) => n.elements.to_vec(),
        AstNode::ObjectExpression(n) => n.properties.to_vec(),
        AstNode::ObjectProperty(n) => vec![n.key, n.value],
        AstNode::SpreadElement(n) => vec![n.argument],
        AstNode::ArrowFunctionExpression(n) => {
            let mut c: Vec<NodeId> = n.params.to_vec();
            c.push(n.body);
            c
        }
        AstNode::AwaitExpression(n) => vec![n.argument],
        AstNode::StaticMemberExpression(n) => vec![n.object],
        AstNode::ComputedMemberExpression(n) => vec![n.object, n.expression],
        AstNode::ChainExpression(n) => vec![n.expression],
        AstNode::ArrayPattern(n) => {
            let mut c: Vec<NodeId> = n.elements.iter().filter_map(|e| *e).collect();
            if let Some(rest) = n.rest {
                c.push(rest);
            }
            c
        }
        AstNode::ObjectPattern(n) => {
            let mut c = n.properties.to_vec();
            if let Some(rest) = n.rest {
                c.push(rest);
            }
            c
        }
        AstNode::ImportDeclaration(n) => n.specifiers.to_vec(),
        AstNode::ExportNamedDeclaration(n) => {
            let mut c = Vec::new();
            if let Some(decl) = n.declaration {
                c.push(decl);
            }
            c.extend_from_slice(&n.specifiers);
            c
        }
        AstNode::ExportDefaultDeclaration(n) => vec![n.declaration],
        AstNode::MethodDefinition(n) => vec![n.key, n.value],
        AstNode::PropertyDefinition(n) => {
            let mut c = vec![n.key];
            if let Some(v) = n.value {
                c.push(v);
            }
            c
        }
        AstNode::JSXElement(n) => {
            let mut c = vec![n.opening_element];
            c.extend_from_slice(&n.children);
            c
        }
        AstNode::JSXOpeningElement(n) => n.attributes.to_vec(),
        AstNode::JSXFragment(n) => n.children.to_vec(),
        AstNode::JSXAttribute(n) => n.value.into_iter().collect(),
        AstNode::JSXSpreadAttribute(n) => vec![n.argument],
        AstNode::JSXExpressionContainer(n) => n.expression.into_iter().collect(),
        AstNode::TSTypeAliasDeclaration(n) => {
            let mut c = vec![n.id];
            c.extend_from_slice(&n.type_parameters);
            c
        }
        AstNode::TSInterfaceDeclaration(n) => {
            let mut c = vec![n.id];
            c.extend_from_slice(&n.body);
            c
        }
        AstNode::TSEnumDeclaration(n) => {
            let mut c = vec![n.id];
            c.extend_from_slice(&n.members);
            c
        }
        AstNode::TSEnumMember(n) => {
            let mut c = vec![n.id];
            if let Some(init) = n.initializer {
                c.push(init);
            }
            c
        }
        AstNode::TSModuleDeclaration(n) => {
            let mut c = vec![n.id];
            if let Some(body) = n.body {
                c.push(body);
            }
            c
        }
        AstNode::TSAsExpression(n) => vec![n.expression],
        AstNode::TSTypeAssertion(n) => vec![n.expression],
        AstNode::TSNonNullExpression(n) => vec![n.expression],
        AstNode::TSTypeLiteral(n) => n.members.to_vec(),
        AstNode::TSTypeReference(n) => n.type_arguments.to_vec(),
        AstNode::TSTypeParameter(n) => {
            let mut c = Vec::new();
            if let Some(constraint) = n.constraint {
                c.push(constraint);
            }
            if let Some(default) = n.default {
                c.push(default);
            }
            c
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AstTree;
    use crate::node::{
        AstNode, BlockStatementNode, DebuggerStatementNode, ExpressionStatementNode,
        IdentifierReferenceNode, ProgramNode,
    };
    use crate::types::{NodeId, Span};

    /// Build a minimal tree: `Program` > `BlockStatement` > `ExpressionStatement` > `IdentifierReference`.
    fn make_test_tree() -> AstTree {
        let mut tree = AstTree::with_capacity(5);

        // 0: Program (body: [1])
        let program_id = tree.push(
            AstNode::Program(ProgramNode {
                span: Span::new(0, 50),
                is_module: false,
                body: Box::new([NodeId(1)]),
            }),
            None,
        );

        // 1: BlockStatement (body: [2])
        let block_id = tree.push(
            AstNode::BlockStatement(BlockStatementNode {
                span: Span::new(0, 50),
                body: Box::new([NodeId(2)]),
            }),
            Some(program_id),
        );

        // 2: ExpressionStatement (expression: 3)
        let expr_stmt_id = tree.push(
            AstNode::ExpressionStatement(ExpressionStatementNode {
                span: Span::new(2, 7),
                expression: NodeId(3),
            }),
            Some(block_id),
        );

        // 3: IdentifierReference
        let _ident_id = tree.push(
            AstNode::IdentifierReference(IdentifierReferenceNode {
                span: Span::new(2, 5),
                name: "foo".to_owned(),
            }),
            Some(expr_stmt_id),
        );

        tree
    }

    #[test]
    fn tree_basics() {
        let tree = make_test_tree();
        assert_eq!(tree.len(), 4, "should have 4 nodes");
        assert!(!tree.is_empty(), "should not be empty");
    }

    #[test]
    fn tree_parent_navigation() {
        let tree = make_test_tree();
        assert!(tree.parent(NodeId::ROOT).is_none(), "root has no parent");
        assert_eq!(
            tree.parent(NodeId(1)),
            Some(NodeId::ROOT),
            "block parent is root"
        );
        assert_eq!(
            tree.parent(NodeId(3)),
            Some(NodeId(2)),
            "ident parent is expr_stmt"
        );
    }

    #[test]
    fn tree_children() {
        let tree = make_test_tree();
        let root_children = tree.children(NodeId::ROOT);
        assert_eq!(root_children, vec![NodeId(1)], "root has one child");

        let block_children = tree.children(NodeId(1));
        assert_eq!(block_children, vec![NodeId(2)], "block has one child");
    }

    #[test]
    fn tree_ancestors() {
        let tree = make_test_tree();
        let ancestors = tree.ancestors(NodeId(3));
        assert_eq!(
            ancestors,
            vec![NodeId(2), NodeId(1), NodeId::ROOT],
            "ancestors from ident to root"
        );
    }

    #[test]
    fn tree_span() {
        let tree = make_test_tree();
        assert_eq!(tree.span(NodeId::ROOT), Some(Span::new(0, 50)), "root span");
    }

    #[test]
    fn empty_tree() {
        let tree = AstTree::new();
        assert!(tree.is_empty(), "empty tree");
        assert_eq!(tree.len(), 0, "zero nodes");
        assert!(tree.get(NodeId::ROOT).is_none(), "no root node");
    }

    #[test]
    fn leaf_node_no_children() {
        let mut tree = AstTree::new();
        tree.push(
            AstNode::DebuggerStatement(DebuggerStatementNode {
                span: Span::new(0, 8),
            }),
            None,
        );
        assert!(
            tree.children(NodeId::ROOT).is_empty(),
            "debugger has no children"
        );
    }
}
