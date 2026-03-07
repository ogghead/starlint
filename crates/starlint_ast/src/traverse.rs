//! Depth-first AST traversal utilities.
//!
//! [`TreeCursor`] walks an [`AstTree`] in depth-first preorder, calling
//! [`TreeVisitor`] methods on enter and leave of each node.

use crate::node::AstNode;
use crate::tree::AstTree;
use crate::types::NodeId;

/// Visitor trait for depth-first traversal of an [`AstTree`].
///
/// Implement this to receive callbacks as nodes are entered and left.
/// Return `false` from `enter_node` to skip the subtree.
#[allow(unused_variables)]
pub trait TreeVisitor {
    /// Called when entering a node (preorder). Return `false` to skip children.
    fn enter_node(&mut self, id: NodeId, node: &AstNode) -> bool {
        true
    }

    /// Called when leaving a node (postorder).
    fn leave_node(&mut self, id: NodeId, node: &AstNode) {}
}

/// A cursor for depth-first traversal of an [`AstTree`].
///
/// Visits nodes in depth-first order, calling `enter_node` on descent and
/// `leave_node` on ascent.
pub struct TreeCursor<'a> {
    /// The tree being traversed.
    tree: &'a AstTree,
}

impl<'a> TreeCursor<'a> {
    /// Create a cursor for the given tree.
    #[must_use]
    pub const fn new(tree: &'a AstTree) -> Self {
        Self { tree }
    }

    /// Run a depth-first traversal starting from `root`, calling visitor methods.
    pub fn walk(&self, root: NodeId, visitor: &mut impl TreeVisitor) {
        if let Some(node) = self.tree.get(root) {
            self.walk_node(root, node, visitor);
        }
    }

    /// Walk starting from the tree root (`NodeId::ROOT`).
    pub fn walk_root(&self, visitor: &mut impl TreeVisitor) {
        self.walk(NodeId::ROOT, visitor);
    }

    /// Recursive walk implementation.
    fn walk_node(&self, id: NodeId, node: &AstNode, visitor: &mut impl TreeVisitor) {
        let descend = visitor.enter_node(id, node);

        if descend {
            for &child_id in self.tree.children(id) {
                if let Some(child_node) = self.tree.get(child_id) {
                    self.walk_node(child_id, child_node, visitor);
                }
            }
        }

        visitor.leave_node(id, node);
    }
}

/// Walk a tree depth-first from the root, calling the visitor.
///
/// Convenience function wrapping [`TreeCursor`].
pub fn walk_tree(tree: &AstTree, visitor: &mut impl TreeVisitor) {
    let cursor = TreeCursor::new(tree);
    cursor.walk_root(visitor);
}

#[cfg(test)]
mod tests {
    use super::{TreeVisitor, walk_tree};
    use crate::node::{
        AstNode, BlockStatementNode, ExpressionStatementNode, IdentifierReferenceNode, ProgramNode,
    };
    use crate::node_type::AstNodeType;
    use crate::tree::AstTree;
    use crate::types::{NodeId, Span};

    /// Visitor that records enter/leave events.
    struct RecordingVisitor {
        /// Events as `("enter"|"leave", AstNodeType)`.
        events: Vec<(&'static str, AstNodeType)>,
    }

    impl RecordingVisitor {
        /// Create a new recording visitor.
        fn new() -> Self {
            Self { events: Vec::new() }
        }
    }

    impl TreeVisitor for RecordingVisitor {
        fn enter_node(&mut self, _id: NodeId, node: &AstNode) -> bool {
            self.events.push(("enter", AstNodeType::from(node)));
            true
        }

        fn leave_node(&mut self, _id: NodeId, node: &AstNode) {
            self.events.push(("leave", AstNodeType::from(node)));
        }
    }

    /// Visitor that skips subtrees when it encounters a `BlockStatement`.
    struct SkippingVisitor {
        /// Entered node types.
        entered: Vec<AstNodeType>,
    }

    impl SkippingVisitor {
        /// Create a new visitor.
        fn new() -> Self {
            Self {
                entered: Vec::new(),
            }
        }
    }

    impl TreeVisitor for SkippingVisitor {
        fn enter_node(&mut self, _id: NodeId, node: &AstNode) -> bool {
            let ty = AstNodeType::from(node);
            self.entered.push(ty);
            // Skip children of BlockStatement
            ty != AstNodeType::BlockStatement
        }

        fn leave_node(&mut self, _id: NodeId, _node: &AstNode) {}
    }

    fn make_simple_tree() -> AstTree {
        let mut tree = AstTree::with_capacity(4);

        // 0: Program
        tree.push(
            AstNode::Program(ProgramNode {
                span: Span::new(0, 20),
                is_module: false,
                body: Box::new([NodeId(1)]),
            }),
            None,
        );

        // 1: BlockStatement
        tree.push(
            AstNode::BlockStatement(BlockStatementNode {
                span: Span::new(0, 20),
                body: Box::new([NodeId(2)]),
            }),
            Some(NodeId(0)),
        );

        // 2: ExpressionStatement
        tree.push(
            AstNode::ExpressionStatement(ExpressionStatementNode {
                span: Span::new(2, 5),
                expression: NodeId(3),
            }),
            Some(NodeId(1)),
        );

        // 3: IdentifierReference
        tree.push(
            AstNode::IdentifierReference(IdentifierReferenceNode {
                span: Span::new(2, 5),
                name: "x".to_owned(),
            }),
            Some(NodeId(2)),
        );

        tree
    }

    #[test]
    fn depth_first_order() {
        let tree = make_simple_tree();
        let mut visitor = RecordingVisitor::new();
        walk_tree(&tree, &mut visitor);

        let expected = vec![
            ("enter", AstNodeType::Program),
            ("enter", AstNodeType::BlockStatement),
            ("enter", AstNodeType::ExpressionStatement),
            ("enter", AstNodeType::IdentifierReference),
            ("leave", AstNodeType::IdentifierReference),
            ("leave", AstNodeType::ExpressionStatement),
            ("leave", AstNodeType::BlockStatement),
            ("leave", AstNodeType::Program),
        ];
        assert_eq!(visitor.events, expected, "traversal order should be DFS");
    }

    #[test]
    fn skip_subtree() {
        let tree = make_simple_tree();
        let mut visitor = SkippingVisitor::new();
        walk_tree(&tree, &mut visitor);

        // Should enter Program and BlockStatement but skip Block's children
        assert_eq!(
            visitor.entered,
            vec![AstNodeType::Program, AstNodeType::BlockStatement],
            "should skip children of BlockStatement"
        );
    }
}
