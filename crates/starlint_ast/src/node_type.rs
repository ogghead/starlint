//! Fieldless discriminant enum for dispatch-table indexing.
//!
//! [`AstNodeType`] has one variant per [`AstNode`](crate::AstNode) variant but
//! carries no data. It is `#[repr(u8)]` so it can be used as an array index
//! for O(1) dispatch.

use serde::{Deserialize, Serialize};

use crate::node::AstNode;

/// Fieldless discriminant matching [`AstNode`] variants.
///
/// Used for dispatch tables and interest-based filtering.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
pub enum AstNodeType {
    // Program
    Program = 0,

    // Statements
    BlockStatement = 1,
    IfStatement = 2,
    SwitchStatement = 3,
    SwitchCase = 4,
    ForStatement = 5,
    ForInStatement = 6,
    ForOfStatement = 7,
    WhileStatement = 8,
    DoWhileStatement = 9,
    TryStatement = 10,
    CatchClause = 11,
    ThrowStatement = 12,
    ReturnStatement = 13,
    LabeledStatement = 14,
    ContinueStatement = 15,
    EmptyStatement = 16,
    WithStatement = 17,
    ExpressionStatement = 18,

    // Declarations
    VariableDeclaration = 19,
    VariableDeclarator = 20,
    Function = 21,
    FunctionBody = 22,
    Class = 23,
    StaticBlock = 24,

    // Expressions
    CallExpression = 25,
    NewExpression = 26,
    BinaryExpression = 27,
    LogicalExpression = 28,
    AssignmentExpression = 29,
    UnaryExpression = 30,
    UpdateExpression = 31,
    ConditionalExpression = 32,
    SequenceExpression = 33,
    IdentifierReference = 34,
    BindingIdentifier = 35,
    StringLiteral = 36,
    NumericLiteral = 37,
    NullLiteral = 38,
    RegExpLiteral = 39,
    TemplateLiteral = 40,
    TaggedTemplateExpression = 41,
    ArrayExpression = 42,
    ObjectExpression = 43,
    ObjectProperty = 44,
    SpreadElement = 45,
    ArrowFunctionExpression = 46,
    AwaitExpression = 47,
    StaticMemberExpression = 48,
    ComputedMemberExpression = 49,
    ChainExpression = 50,
    ThisExpression = 51,
    DebuggerStatement = 52,

    // Patterns
    ArrayPattern = 53,
    ObjectPattern = 54,

    // Modules
    ImportDeclaration = 55,
    ImportSpecifier = 56,
    ExportNamedDeclaration = 57,
    ExportDefaultDeclaration = 58,
    ExportAllDeclaration = 59,
    ExportSpecifier = 60,

    // Class members
    MethodDefinition = 61,
    PropertyDefinition = 62,

    // JSX
    JSXElement = 63,
    JSXOpeningElement = 64,
    JSXFragment = 65,
    JSXAttribute = 66,
    JSXSpreadAttribute = 67,
    JSXExpressionContainer = 68,
    JSXNamespacedName = 69,
    JSXText = 70,

    // TypeScript
    TSTypeAliasDeclaration = 71,
    TSInterfaceDeclaration = 72,
    TSEnumDeclaration = 73,
    TSEnumMember = 74,
    TSModuleDeclaration = 75,
    TSAsExpression = 76,
    TSTypeAssertion = 77,
    TSNonNullExpression = 78,
    TSTypeLiteral = 79,
    TSTypeReference = 80,
    TSTypeParameter = 81,
    TSAnyKeyword = 82,
    TSVoidKeyword = 83,
}

/// Total number of `AstNodeType` variants (for sizing dispatch tables).
pub const AST_NODE_TYPE_COUNT: usize = 84;

impl From<&AstNode> for AstNodeType {
    fn from(node: &AstNode) -> Self {
        match node {
            AstNode::Program(_) => Self::Program,
            AstNode::BlockStatement(_) => Self::BlockStatement,
            AstNode::IfStatement(_) => Self::IfStatement,
            AstNode::SwitchStatement(_) => Self::SwitchStatement,
            AstNode::SwitchCase(_) => Self::SwitchCase,
            AstNode::ForStatement(_) => Self::ForStatement,
            AstNode::ForInStatement(_) => Self::ForInStatement,
            AstNode::ForOfStatement(_) => Self::ForOfStatement,
            AstNode::WhileStatement(_) => Self::WhileStatement,
            AstNode::DoWhileStatement(_) => Self::DoWhileStatement,
            AstNode::TryStatement(_) => Self::TryStatement,
            AstNode::CatchClause(_) => Self::CatchClause,
            AstNode::ThrowStatement(_) => Self::ThrowStatement,
            AstNode::ReturnStatement(_) => Self::ReturnStatement,
            AstNode::LabeledStatement(_) => Self::LabeledStatement,
            AstNode::ContinueStatement(_) => Self::ContinueStatement,
            AstNode::EmptyStatement(_) => Self::EmptyStatement,
            AstNode::WithStatement(_) => Self::WithStatement,
            AstNode::ExpressionStatement(_) => Self::ExpressionStatement,
            AstNode::VariableDeclaration(_) => Self::VariableDeclaration,
            AstNode::VariableDeclarator(_) => Self::VariableDeclarator,
            AstNode::Function(_) => Self::Function,
            AstNode::FunctionBody(_) => Self::FunctionBody,
            AstNode::Class(_) => Self::Class,
            AstNode::StaticBlock(_) => Self::StaticBlock,
            AstNode::CallExpression(_) => Self::CallExpression,
            AstNode::NewExpression(_) => Self::NewExpression,
            AstNode::BinaryExpression(_) => Self::BinaryExpression,
            AstNode::LogicalExpression(_) => Self::LogicalExpression,
            AstNode::AssignmentExpression(_) => Self::AssignmentExpression,
            AstNode::UnaryExpression(_) => Self::UnaryExpression,
            AstNode::UpdateExpression(_) => Self::UpdateExpression,
            AstNode::ConditionalExpression(_) => Self::ConditionalExpression,
            AstNode::SequenceExpression(_) => Self::SequenceExpression,
            AstNode::IdentifierReference(_) => Self::IdentifierReference,
            AstNode::BindingIdentifier(_) => Self::BindingIdentifier,
            AstNode::StringLiteral(_) => Self::StringLiteral,
            AstNode::NumericLiteral(_) => Self::NumericLiteral,
            AstNode::NullLiteral(_) => Self::NullLiteral,
            AstNode::RegExpLiteral(_) => Self::RegExpLiteral,
            AstNode::TemplateLiteral(_) => Self::TemplateLiteral,
            AstNode::TaggedTemplateExpression(_) => Self::TaggedTemplateExpression,
            AstNode::ArrayExpression(_) => Self::ArrayExpression,
            AstNode::ObjectExpression(_) => Self::ObjectExpression,
            AstNode::ObjectProperty(_) => Self::ObjectProperty,
            AstNode::SpreadElement(_) => Self::SpreadElement,
            AstNode::ArrowFunctionExpression(_) => Self::ArrowFunctionExpression,
            AstNode::AwaitExpression(_) => Self::AwaitExpression,
            AstNode::StaticMemberExpression(_) => Self::StaticMemberExpression,
            AstNode::ComputedMemberExpression(_) => Self::ComputedMemberExpression,
            AstNode::ChainExpression(_) => Self::ChainExpression,
            AstNode::ThisExpression(_) => Self::ThisExpression,
            AstNode::DebuggerStatement(_) => Self::DebuggerStatement,
            AstNode::ArrayPattern(_) => Self::ArrayPattern,
            AstNode::ObjectPattern(_) => Self::ObjectPattern,
            AstNode::ImportDeclaration(_) => Self::ImportDeclaration,
            AstNode::ImportSpecifier(_) => Self::ImportSpecifier,
            AstNode::ExportNamedDeclaration(_) => Self::ExportNamedDeclaration,
            AstNode::ExportDefaultDeclaration(_) => Self::ExportDefaultDeclaration,
            AstNode::ExportAllDeclaration(_) => Self::ExportAllDeclaration,
            AstNode::ExportSpecifier(_) => Self::ExportSpecifier,
            AstNode::MethodDefinition(_) => Self::MethodDefinition,
            AstNode::PropertyDefinition(_) => Self::PropertyDefinition,
            AstNode::JSXElement(_) => Self::JSXElement,
            AstNode::JSXOpeningElement(_) => Self::JSXOpeningElement,
            AstNode::JSXFragment(_) => Self::JSXFragment,
            AstNode::JSXAttribute(_) => Self::JSXAttribute,
            AstNode::JSXSpreadAttribute(_) => Self::JSXSpreadAttribute,
            AstNode::JSXExpressionContainer(_) => Self::JSXExpressionContainer,
            AstNode::JSXNamespacedName(_) => Self::JSXNamespacedName,
            AstNode::JSXText(_) => Self::JSXText,
            AstNode::TSTypeAliasDeclaration(_) => Self::TSTypeAliasDeclaration,
            AstNode::TSInterfaceDeclaration(_) => Self::TSInterfaceDeclaration,
            AstNode::TSEnumDeclaration(_) => Self::TSEnumDeclaration,
            AstNode::TSEnumMember(_) => Self::TSEnumMember,
            AstNode::TSModuleDeclaration(_) => Self::TSModuleDeclaration,
            AstNode::TSAsExpression(_) => Self::TSAsExpression,
            AstNode::TSTypeAssertion(_) => Self::TSTypeAssertion,
            AstNode::TSNonNullExpression(_) => Self::TSNonNullExpression,
            AstNode::TSTypeLiteral(_) => Self::TSTypeLiteral,
            AstNode::TSTypeReference(_) => Self::TSTypeReference,
            AstNode::TSTypeParameter(_) => Self::TSTypeParameter,
            AstNode::TSAnyKeyword(_) => Self::TSAnyKeyword,
            AstNode::TSVoidKeyword(_) => Self::TSVoidKeyword,
        }
    }
}

impl AstNodeType {
    /// Convert to `usize` for use as an array/dispatch-table index.
    #[must_use]
    #[allow(clippy::as_conversions)]
    pub const fn index(self) -> usize {
        self as usize
    }
}

#[cfg(test)]
mod tests {
    use super::{AST_NODE_TYPE_COUNT, AstNodeType};
    use crate::node::{AstNode, DebuggerStatementNode};
    use crate::types::Span;

    #[test]
    fn discriminant_matches() {
        let node = AstNode::DebuggerStatement(DebuggerStatementNode { span: Span::EMPTY });
        let ty: AstNodeType = (&node).into();
        assert_eq!(ty, AstNodeType::DebuggerStatement, "type should match");
    }

    #[test]
    fn count_is_correct() {
        // TSVoidKeyword is the last variant at index 83, so count should be 84.
        assert_eq!(
            AstNodeType::TSVoidKeyword.index() + 1,
            AST_NODE_TYPE_COUNT,
            "count should match last discriminant + 1"
        );
    }
}
