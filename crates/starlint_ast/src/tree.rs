//! Flat indexed AST container.
//!
//! [`AstTree`] owns all nodes in a contiguous `Vec`, with parent pointers
//! stored in a parallel `Vec`. Child references within nodes use [`NodeId`]
//! indices.

use crate::node::AstNode;
use crate::node_type::AstNodeType;
use crate::types::{NodeId, Span};

/// A flat indexed AST. Nodes reference children by [`NodeId`] index.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AstTree {
    /// All nodes in depth-first preorder.
    nodes: Vec<AstNode>,
    /// Parent of each node (parallel to `nodes`). `None` for root.
    parents: Vec<Option<NodeId>>,
    /// Children of each node (parallel to `nodes`). Maintained during `push()`.
    /// Derived from `parents`, so skipped during serialization.
    #[serde(skip)]
    children: Vec<Vec<NodeId>>,
}

impl AstTree {
    /// Create a new empty tree.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            nodes: Vec::new(),
            parents: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Create a tree with pre-allocated capacity.
    #[must_use]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(cap),
            parents: Vec::with_capacity(cap),
            children: Vec::with_capacity(cap),
        }
    }

    /// Push a node, returning its [`NodeId`].
    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
    pub fn push(&mut self, node: AstNode, parent: Option<NodeId>) -> NodeId {
        let id = NodeId(self.nodes.len() as u32);
        self.nodes.push(node);
        self.parents.push(parent);
        self.children.push(Vec::new());
        // Register this node as a child of its parent.
        if let Some(pid) = parent {
            if let Some(siblings) = self.children.get_mut(pid.index()) {
                siblings.push(id);
            }
        }
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

    /// Get the direct children of a node.
    ///
    /// Returns a slice of child [`NodeId`]s in insertion order. O(1) lookup
    /// via the pre-computed children index.
    #[must_use]
    pub fn children(&self, id: NodeId) -> &[NodeId] {
        self.children.get(id.index()).map_or(&[], Vec::as_slice)
    }

    /// Reserve a slot in the tree, returning its [`NodeId`].
    ///
    /// The slot is filled with a placeholder `Unknown` node. Call [`set`](Self::set)
    /// to replace it with the real node after children have been pushed.
    pub fn reserve(&mut self, parent: Option<NodeId>) -> NodeId {
        self.push(
            AstNode::Unknown(crate::node::UnknownNode {
                span: crate::types::Span::EMPTY,
            }),
            parent,
        )
    }

    /// Replace a previously reserved or pushed node at the given ID.
    ///
    /// # Panics
    ///
    /// Panics if `id` is out of bounds.
    #[allow(clippy::indexing_slicing)]
    pub fn set(&mut self, id: NodeId, node: AstNode) {
        self.nodes[id.index()] = node;
    }

    /// Return the [`NodeId`] that will be assigned to the next pushed node.
    #[must_use]
    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
    pub fn next_id(&self) -> NodeId {
        NodeId(self.nodes.len() as u32)
    }

    /// Collect all [`BindingIdentifierNode`](crate::node::BindingIdentifierNode)
    /// references in the subtree rooted at `id`.
    ///
    /// Walks children recursively (useful for destructuring patterns) and
    /// returns `(NodeId, &BindingIdentifierNode)` pairs.
    #[must_use]
    pub fn get_binding_identifiers(
        &self,
        id: NodeId,
    ) -> Vec<(NodeId, &crate::node::BindingIdentifierNode)> {
        let mut result = Vec::new();
        self.collect_binding_identifiers(id, &mut result);
        result
    }

    /// Recursive helper for [`get_binding_identifiers`](Self::get_binding_identifiers).
    fn collect_binding_identifiers<'a>(
        &'a self,
        id: NodeId,
        out: &mut Vec<(NodeId, &'a crate::node::BindingIdentifierNode)>,
    ) {
        let Some(node) = self.get(id) else {
            return;
        };
        if let Some(binding) = node.as_binding_identifier() {
            out.push((id, binding));
            return;
        }
        for child_id in node_children(node) {
            self.collect_binding_identifiers(child_id, out);
        }
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

    /// Rebuild the children index from the parent array.
    ///
    /// Call this after deserialization (the `children` field is `serde(skip)`).
    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
    pub fn rebuild_children_index(&mut self) {
        self.children = vec![Vec::new(); self.nodes.len()];
        for (i, parent) in self.parents.iter().enumerate() {
            if let Some(pid) = parent {
                if let Some(siblings) = self.children.get_mut(pid.index()) {
                    siblings.push(NodeId(i as u32));
                }
            }
        }
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
            if let Some(ta) = n.type_annotation {
                c.push(ta);
            }
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
            c.extend_from_slice(&n.type_parameters);
            c.extend_from_slice(&n.params);
            if let Some(rt) = n.return_type {
                c.push(rt);
            }
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
        AstNode::BreakStatement(_)
        | AstNode::ContinueStatement(_)
        | AstNode::BooleanLiteral(_)
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
        | AstNode::JSXNamespacedName(_)
        | AstNode::Unknown(_) => Vec::new(),
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
        AstNode::AssignmentPattern(n) => vec![n.left, n.right],
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
            if let Some(ta) = n.type_annotation {
                c.push(ta);
            }
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
        AstNode, BlockStatementNode, DebuggerStatementNode, EmptyStatementNode,
        ExpressionStatementNode, IdentifierReferenceNode, ProgramNode,
    };
    use crate::node_type::AstNodeType;
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

    /// Helper: create a program node with no body.
    fn make_program_node(span: Span) -> AstNode {
        AstNode::Program(ProgramNode {
            span,
            is_module: false,
            body: Box::new([]),
        })
    }

    /// Helper: create an empty-statement node.
    fn make_empty_stmt(span: Span) -> AstNode {
        AstNode::EmptyStatement(EmptyStatementNode { span })
    }

    // -----------------------------------------------------------------------
    // Existing tests
    // -----------------------------------------------------------------------

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

    // -----------------------------------------------------------------------
    // New comprehensive tests
    // -----------------------------------------------------------------------

    #[test]
    fn empty_tree_properties() {
        let tree = AstTree::new();
        assert!(tree.is_empty(), "new tree should be empty");
        assert_eq!(tree.len(), 0, "new tree should have 0 nodes");
        assert!(
            tree.node(NodeId::ROOT).is_none(),
            "new tree should have no root node"
        );
        assert!(
            tree.parent(NodeId::ROOT).is_none(),
            "new tree should have no parent for root"
        );
        assert!(
            tree.children(NodeId::ROOT).is_empty(),
            "new tree should have empty children for root"
        );
        assert!(
            tree.node_type(NodeId::ROOT).is_none(),
            "new tree should have no node type for root"
        );
        assert!(
            tree.span(NodeId::ROOT).is_none(),
            "new tree should have no span for root"
        );
        assert_eq!(
            tree.next_id(),
            NodeId(0),
            "next_id on empty tree should be 0"
        );
        assert_eq!(
            tree.iter().count(),
            0,
            "iter on empty tree should yield nothing"
        );
    }

    #[test]
    fn single_node_tree() {
        let mut tree = AstTree::new();
        let id = tree.push(make_program_node(Span::new(0, 100)), None);

        assert_eq!(id, NodeId(0), "first node should be at index 0");
        assert_eq!(tree.len(), 1, "tree should have 1 node");
        assert!(!tree.is_empty(), "tree should not be empty after push");

        // The single node is both root and leaf.
        assert!(
            tree.node(id).is_some(),
            "should be able to retrieve the single node"
        );
        assert!(tree.parent(id).is_none(), "root node should have no parent");
        assert!(
            tree.children(id).is_empty(),
            "root with no pushed children should have empty children"
        );
    }

    #[test]
    fn parent_child_relationship() {
        let mut tree = AstTree::new();
        let root_id = tree.push(make_program_node(Span::new(0, 50)), None);
        let child_id = tree.push(make_empty_stmt(Span::new(0, 1)), Some(root_id));

        assert_eq!(
            tree.parent(child_id),
            Some(root_id),
            "child should report root as parent"
        );
        assert_eq!(
            tree.children(root_id),
            &[child_id],
            "root should have one child"
        );
        assert!(
            tree.children(child_id).is_empty(),
            "child (leaf) should have no children"
        );
    }

    #[test]
    fn multiple_children() {
        let mut tree = AstTree::new();
        let root_id = tree.push(make_program_node(Span::new(0, 100)), None);
        let c1 = tree.push(make_empty_stmt(Span::new(0, 1)), Some(root_id));
        let c2 = tree.push(make_empty_stmt(Span::new(2, 3)), Some(root_id));
        let c3 = tree.push(make_empty_stmt(Span::new(4, 5)), Some(root_id));

        assert_eq!(
            tree.children(root_id),
            &[c1, c2, c3],
            "root should have three children in insertion order"
        );
        for child in &[c1, c2, c3] {
            assert_eq!(
                tree.parent(*child),
                Some(root_id),
                "each child should have root as parent"
            );
        }
    }

    #[test]
    fn get_and_node_are_equivalent() {
        let mut tree = AstTree::new();
        let id = tree.push(make_empty_stmt(Span::new(0, 1)), None);

        let via_get = tree.get(id);
        let via_node = tree.node(id);
        assert!(via_get.is_some(), "get should return Some for valid id");
        assert!(via_node.is_some(), "node should return Some for valid id");

        // Both should return the same span (proving they're the same node).
        assert_eq!(
            via_get.map(AstNode::span),
            via_node.map(AstNode::span),
            "get() and node() should return identical nodes"
        );
    }

    #[test]
    fn node_type_returns_correct_variant() {
        let mut tree = AstTree::new();
        let prog_id = tree.push(make_program_node(Span::new(0, 100)), None);
        let empty_id = tree.push(make_empty_stmt(Span::new(0, 1)), Some(prog_id));
        let dbg_id = tree.push(
            AstNode::DebuggerStatement(DebuggerStatementNode {
                span: Span::new(2, 10),
            }),
            Some(prog_id),
        );

        assert_eq!(
            tree.node_type(prog_id),
            Some(AstNodeType::Program),
            "program node should have Program type"
        );
        assert_eq!(
            tree.node_type(empty_id),
            Some(AstNodeType::EmptyStatement),
            "empty statement should have EmptyStatement type"
        );
        assert_eq!(
            tree.node_type(dbg_id),
            Some(AstNodeType::DebuggerStatement),
            "debugger statement should have DebuggerStatement type"
        );
    }

    #[test]
    fn node_type_returns_none_for_invalid_id() {
        let tree = AstTree::new();
        assert!(
            tree.node_type(NodeId(999)).is_none(),
            "node_type should return None for out-of-bounds id"
        );
    }

    #[test]
    fn out_of_bounds_access_returns_none() {
        let mut tree = AstTree::new();
        tree.push(make_empty_stmt(Span::new(0, 1)), None);

        assert!(
            tree.node(NodeId(999)).is_none(),
            "node() should return None for large out-of-bounds id"
        );
        assert!(
            tree.get(NodeId(999)).is_none(),
            "get() should return None for large out-of-bounds id"
        );
        assert!(
            tree.parent(NodeId(999)).is_none(),
            "parent() should return None for out-of-bounds id"
        );
        assert!(
            tree.span(NodeId(999)).is_none(),
            "span() should return None for out-of-bounds id"
        );
        assert!(
            tree.children(NodeId(999)).is_empty(),
            "children() should return empty slice for out-of-bounds id"
        );
    }

    #[test]
    fn next_id_increments_after_push() {
        let mut tree = AstTree::new();
        assert_eq!(
            tree.next_id(),
            NodeId(0),
            "next_id should be 0 before any push"
        );

        tree.push(make_empty_stmt(Span::new(0, 1)), None);
        assert_eq!(
            tree.next_id(),
            NodeId(1),
            "next_id should be 1 after first push"
        );

        tree.push(make_empty_stmt(Span::new(2, 3)), Some(NodeId(0)));
        assert_eq!(
            tree.next_id(),
            NodeId(2),
            "next_id should be 2 after second push"
        );

        tree.push(make_empty_stmt(Span::new(4, 5)), Some(NodeId(0)));
        assert_eq!(
            tree.next_id(),
            NodeId(3),
            "next_id should be 3 after third push"
        );
    }

    #[test]
    #[allow(clippy::indexing_slicing)]
    fn iter_returns_all_nodes_in_order() {
        let mut tree = AstTree::new();
        let id0 = tree.push(make_program_node(Span::new(0, 100)), None);
        let id1 = tree.push(make_empty_stmt(Span::new(0, 1)), Some(id0));
        let id2 = tree.push(make_empty_stmt(Span::new(2, 3)), Some(id0));

        let collected: Vec<(NodeId, &AstNode)> = tree.iter().collect();
        assert_eq!(collected.len(), 3, "iter should yield 3 nodes");

        assert_eq!(collected[0].0, id0, "first iter element should have id 0");
        assert_eq!(collected[1].0, id1, "second iter element should have id 1");
        assert_eq!(collected[2].0, id2, "third iter element should have id 2");

        // Verify the types match what we pushed.
        assert_eq!(
            AstNodeType::from(collected[0].1),
            AstNodeType::Program,
            "first node should be a Program"
        );
        assert_eq!(
            AstNodeType::from(collected[1].1),
            AstNodeType::EmptyStatement,
            "second node should be an EmptyStatement"
        );
        assert_eq!(
            AstNodeType::from(collected[2].1),
            AstNodeType::EmptyStatement,
            "third node should be an EmptyStatement"
        );
    }

    #[test]
    fn iter_empty_tree() {
        let tree = AstTree::new();
        assert_eq!(
            tree.iter().count(),
            0,
            "iter on empty tree should yield no elements"
        );
    }

    #[test]
    #[allow(clippy::panic)]
    fn serialization_roundtrip() {
        let tree = make_test_tree();
        let Ok(json) = serde_json::to_string(&tree) else {
            panic!("tree should serialize to JSON without error");
        };
        let Ok(mut deserialized) = serde_json::from_str::<AstTree>(&json) else {
            panic!("tree should deserialize from JSON without error");
        };

        assert_eq!(
            deserialized.len(),
            tree.len(),
            "deserialized tree should have same number of nodes"
        );

        // Children index is serde(skip), so it should be empty after deserialization.
        assert!(
            deserialized.children(NodeId::ROOT).is_empty(),
            "children should be empty after deserialization (serde skip)"
        );

        // Rebuild children index and verify it matches.
        deserialized.rebuild_children_index();
        assert_eq!(
            deserialized.children(NodeId::ROOT),
            tree.children(NodeId::ROOT),
            "rebuilt children should match original for root"
        );
        assert_eq!(
            deserialized.children(NodeId(1)),
            tree.children(NodeId(1)),
            "rebuilt children should match original for block"
        );
        assert_eq!(
            deserialized.children(NodeId(2)),
            tree.children(NodeId(2)),
            "rebuilt children should match original for expr_stmt"
        );
    }

    #[test]
    fn rebuild_children_index_restores_relationships() {
        let mut tree = AstTree::new();
        let root = tree.push(make_program_node(Span::new(0, 50)), None);
        let c1 = tree.push(make_empty_stmt(Span::new(0, 1)), Some(root));
        let c2 = tree.push(make_empty_stmt(Span::new(2, 3)), Some(root));
        let gc = tree.push(make_empty_stmt(Span::new(0, 1)), Some(c1));

        // Snapshot the expected children.
        let root_children: Vec<NodeId> = tree.children(root).to_vec();
        let c1_children: Vec<NodeId> = tree.children(c1).to_vec();
        let c2_children: Vec<NodeId> = tree.children(c2).to_vec();

        // Rebuild and verify.
        tree.rebuild_children_index();

        assert_eq!(
            tree.children(root),
            root_children.as_slice(),
            "root children should be restored after rebuild"
        );
        assert_eq!(
            tree.children(c1),
            c1_children.as_slice(),
            "c1 children should be restored after rebuild"
        );
        assert_eq!(
            tree.children(c2),
            c2_children.as_slice(),
            "c2 children should be restored after rebuild"
        );
        assert!(
            tree.children(gc).is_empty(),
            "grandchild (leaf) should have no children after rebuild"
        );
    }

    #[test]
    fn with_capacity_creates_empty_tree() {
        let tree = AstTree::with_capacity(100);
        assert!(tree.is_empty(), "with_capacity tree should be empty");
        assert_eq!(tree.len(), 0, "with_capacity tree should have 0 nodes");
        assert_eq!(
            tree.next_id(),
            NodeId(0),
            "with_capacity tree next_id should be 0"
        );
    }

    #[test]
    fn default_creates_empty_tree() {
        let tree = AstTree::default();
        assert!(tree.is_empty(), "default tree should be empty");
        assert_eq!(tree.len(), 0, "default tree should have 0 nodes");
    }

    #[test]
    fn reserve_and_set() {
        let mut tree = AstTree::new();
        let reserved_id = tree.reserve(None);

        // Reserved slot should exist as Unknown.
        assert_eq!(tree.len(), 1, "tree should have 1 node after reserve");
        assert_eq!(
            tree.node_type(reserved_id),
            Some(AstNodeType::Unknown),
            "reserved slot should be Unknown"
        );

        // Replace it with a real node.
        tree.set(reserved_id, make_program_node(Span::new(0, 100)));
        assert_eq!(
            tree.node_type(reserved_id),
            Some(AstNodeType::Program),
            "after set, node should be Program"
        );
        assert_eq!(
            tree.span(reserved_id),
            Some(Span::new(0, 100)),
            "after set, span should match"
        );
    }

    #[test]
    fn reserve_preserves_parent_child() {
        let mut tree = AstTree::new();
        let root = tree.push(make_program_node(Span::new(0, 100)), None);
        let reserved = tree.reserve(Some(root));

        assert_eq!(
            tree.parent(reserved),
            Some(root),
            "reserved node should have correct parent"
        );
        assert_eq!(
            tree.children(root),
            &[reserved],
            "parent should list reserved node as child"
        );

        // Set should not alter parent-child relationships.
        tree.set(reserved, make_empty_stmt(Span::new(0, 1)));
        assert_eq!(
            tree.parent(reserved),
            Some(root),
            "parent should be unchanged after set"
        );
        assert_eq!(
            tree.children(root),
            &[reserved],
            "children should be unchanged after set"
        );
    }

    #[test]
    fn ancestors_of_root_is_empty() {
        let mut tree = AstTree::new();
        tree.push(make_program_node(Span::new(0, 50)), None);

        let ancestors = tree.ancestors(NodeId::ROOT);
        assert!(ancestors.is_empty(), "root should have no ancestors");
    }

    #[test]
    fn ancestors_of_invalid_id_is_empty() {
        let tree = AstTree::new();
        let ancestors = tree.ancestors(NodeId(999));
        assert!(
            ancestors.is_empty(),
            "ancestors of invalid node should be empty"
        );
    }

    #[test]
    fn ancestors_chain_depth_three() {
        let mut tree = AstTree::new();
        let root = tree.push(make_program_node(Span::new(0, 100)), None);
        let mid = tree.push(make_empty_stmt(Span::new(0, 1)), Some(root));
        let leaf = tree.push(make_empty_stmt(Span::new(0, 1)), Some(mid));

        let ancestors = tree.ancestors(leaf);
        assert_eq!(
            ancestors,
            vec![mid, root],
            "leaf ancestors should be [mid, root]"
        );
    }

    #[test]
    fn nodes_slice_access() {
        let tree = make_test_tree();
        let nodes = tree.nodes();
        assert_eq!(
            nodes.len(),
            4,
            "nodes() slice should have same length as len()"
        );
        let first = nodes.first();
        assert!(first.is_some(), "nodes slice should have a first element");
        assert_eq!(
            first.map(AstNodeType::from),
            Some(AstNodeType::Program),
            "first node in slice should be Program"
        );
    }

    #[test]
    fn parents_slice_access() {
        let tree = make_test_tree();
        let parents = tree.parents();
        assert_eq!(
            parents.len(),
            4,
            "parents() slice should have same length as len()"
        );
        let first_parent = parents.first().copied().flatten();
        assert!(first_parent.is_none(), "root node parent should be None");
        let second_parent = parents.get(1).copied().flatten();
        assert_eq!(
            second_parent,
            Some(NodeId::ROOT),
            "second node parent should be root"
        );
    }

    #[test]
    fn span_returns_correct_values() {
        let mut tree = AstTree::new();
        let id = tree.push(make_empty_stmt(Span::new(10, 20)), None);
        assert_eq!(
            tree.span(id),
            Some(Span::new(10, 20)),
            "span should return the node's span"
        );
    }

    #[test]
    fn span_returns_none_for_invalid_id() {
        let tree = AstTree::new();
        assert!(
            tree.span(NodeId(42)).is_none(),
            "span should return None for invalid id"
        );
    }

    #[test]
    fn push_returns_sequential_ids() {
        let mut tree = AstTree::new();
        let id0 = tree.push(make_empty_stmt(Span::new(0, 1)), None);
        let id1 = tree.push(make_empty_stmt(Span::new(1, 2)), Some(id0));
        let id2 = tree.push(make_empty_stmt(Span::new(2, 3)), Some(id0));
        let id3 = tree.push(make_empty_stmt(Span::new(3, 4)), Some(id1));

        assert_eq!(id0, NodeId(0), "first push should return id 0");
        assert_eq!(id1, NodeId(1), "second push should return id 1");
        assert_eq!(id2, NodeId(2), "third push should return id 2");
        assert_eq!(id3, NodeId(3), "fourth push should return id 3");
    }

    #[test]
    fn deep_tree_structure() {
        // Build a chain: root -> c1 -> c2 -> c3 -> c4.
        let mut tree = AstTree::new();
        let root = tree.push(make_program_node(Span::new(0, 100)), None);
        let c1 = tree.push(make_empty_stmt(Span::new(0, 1)), Some(root));
        let c2 = tree.push(make_empty_stmt(Span::new(0, 1)), Some(c1));
        let c3 = tree.push(make_empty_stmt(Span::new(0, 1)), Some(c2));
        let c4 = tree.push(make_empty_stmt(Span::new(0, 1)), Some(c3));

        // Verify the chain via parent.
        assert_eq!(tree.parent(c4), Some(c3), "c4 parent should be c3");
        assert_eq!(tree.parent(c3), Some(c2), "c3 parent should be c2");
        assert_eq!(tree.parent(c2), Some(c1), "c2 parent should be c1");

        // Verify ancestors from the leaf.
        let ancestors = tree.ancestors(c4);
        assert_eq!(
            ancestors,
            vec![c3, c2, c1, root],
            "ancestors from c4 should be [c3, c2, c1, root]"
        );

        // Each intermediate node should have exactly one child.
        assert_eq!(tree.children(root), &[c1], "root should have one child: c1");
        assert_eq!(tree.children(c1), &[c2], "c1 should have one child: c2");
        assert_eq!(tree.children(c3), &[c4], "c3 should have one child: c4");
        assert!(
            tree.children(c4).is_empty(),
            "leaf node c4 should have no children"
        );
    }

    #[test]
    fn push_with_invalid_parent_does_not_panic() {
        // Pushing a node with a parent ID that doesn't exist yet should not
        // crash; the parent-child link just won't be recorded.
        let mut tree = AstTree::new();
        let id = tree.push(make_empty_stmt(Span::new(0, 1)), Some(NodeId(999)));
        assert_eq!(tree.len(), 1, "node should still be added");
        assert_eq!(
            tree.parent(id),
            Some(NodeId(999)),
            "parent should be stored even if invalid"
        );
    }

    #[test]
    fn iter_matches_len() {
        let tree = make_test_tree();
        assert_eq!(
            tree.iter().count(),
            tree.len(),
            "iter count should equal len"
        );
    }

    #[test]
    fn children_of_invalid_id_returns_empty_slice() {
        let tree = AstTree::new();
        assert!(
            tree.children(NodeId(0)).is_empty(),
            "children of non-existent id 0 should be empty"
        );
        assert!(
            tree.children(NodeId(u32::MAX)).is_empty(),
            "children of max id should be empty"
        );
    }

    #[test]
    #[allow(clippy::panic)]
    fn serialization_preserves_parent_links() {
        let tree = make_test_tree();
        let Ok(json) = serde_json::to_string(&tree) else {
            panic!("serialization should succeed");
        };
        let Ok(deserialized): Result<AstTree, _> = serde_json::from_str(&json) else {
            panic!("deserialization should succeed");
        };

        // Parent links are serialized (not skipped).
        assert_eq!(
            deserialized.parent(NodeId(1)),
            tree.parent(NodeId(1)),
            "parent links should survive serialization roundtrip"
        );
        assert_eq!(
            deserialized.parent(NodeId(3)),
            tree.parent(NodeId(3)),
            "nested parent link should survive serialization roundtrip"
        );
    }

    #[test]
    #[allow(clippy::panic)]
    fn serialization_preserves_node_types() {
        let tree = make_test_tree();
        let Ok(json) = serde_json::to_string(&tree) else {
            panic!("serialization should succeed");
        };
        let Ok(deserialized): Result<AstTree, _> = serde_json::from_str(&json) else {
            panic!("deserialization should succeed");
        };

        for (id, node) in tree.iter() {
            let original_type = AstNodeType::from(node);
            let deser_type = deserialized.node_type(id);
            assert_eq!(
                deser_type,
                Some(original_type),
                "node type at id {id:?} should survive roundtrip"
            );
        }
    }

    #[test]
    fn rebuild_children_index_on_empty_tree() {
        let mut tree = AstTree::new();
        tree.rebuild_children_index();
        assert!(
            tree.is_empty(),
            "rebuilding empty tree should keep it empty"
        );
    }
}
