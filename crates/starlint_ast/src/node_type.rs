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
    BreakStatement = 15,
    ContinueStatement = 16,
    EmptyStatement = 17,
    WithStatement = 18,
    ExpressionStatement = 19,

    // Declarations
    VariableDeclaration = 20,
    VariableDeclarator = 21,
    Function = 22,
    FunctionBody = 23,
    Class = 24,
    StaticBlock = 25,

    // Expressions
    CallExpression = 26,
    NewExpression = 27,
    BinaryExpression = 28,
    LogicalExpression = 29,
    AssignmentExpression = 30,
    UnaryExpression = 31,
    UpdateExpression = 32,
    ConditionalExpression = 33,
    SequenceExpression = 34,
    IdentifierReference = 35,
    BindingIdentifier = 36,
    StringLiteral = 37,
    NumericLiteral = 38,
    BooleanLiteral = 39,
    NullLiteral = 40,
    RegExpLiteral = 41,
    TemplateLiteral = 42,
    TaggedTemplateExpression = 43,
    ArrayExpression = 44,
    ObjectExpression = 45,
    ObjectProperty = 46,
    SpreadElement = 47,
    ArrowFunctionExpression = 48,
    AwaitExpression = 49,
    StaticMemberExpression = 50,
    ComputedMemberExpression = 51,
    ChainExpression = 52,
    ThisExpression = 53,
    DebuggerStatement = 54,

    // Patterns
    ArrayPattern = 55,
    ObjectPattern = 56,
    AssignmentPattern = 87,

    // Modules
    ImportDeclaration = 57,
    ImportSpecifier = 58,
    ExportNamedDeclaration = 59,
    ExportDefaultDeclaration = 60,
    ExportAllDeclaration = 61,
    ExportSpecifier = 62,

    // Class members
    MethodDefinition = 63,
    PropertyDefinition = 64,

    // JSX
    JSXElement = 65,
    JSXOpeningElement = 66,
    JSXFragment = 67,
    JSXAttribute = 68,
    JSXSpreadAttribute = 69,
    JSXExpressionContainer = 70,
    JSXNamespacedName = 71,
    JSXText = 72,

    // TypeScript
    TSTypeAliasDeclaration = 73,
    TSInterfaceDeclaration = 74,
    TSEnumDeclaration = 75,
    TSEnumMember = 76,
    TSModuleDeclaration = 77,
    TSAsExpression = 78,
    TSTypeAssertion = 79,
    TSNonNullExpression = 80,
    TSTypeLiteral = 81,
    TSTypeReference = 82,
    TSTypeParameter = 83,
    TSAnyKeyword = 84,
    TSVoidKeyword = 85,
    Unknown = 88,
}

/// Total number of `AstNodeType` variants (for sizing dispatch tables).
pub const AST_NODE_TYPE_COUNT: usize = 89;

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
            AstNode::BreakStatement(_) => Self::BreakStatement,
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
            AstNode::BooleanLiteral(_) => Self::BooleanLiteral,
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
            AstNode::AssignmentPattern(_) => Self::AssignmentPattern,
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
            AstNode::Unknown(_) => Self::Unknown,
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
        // Unknown is the last variant at index 86, so count should be 87.
        assert_eq!(
            AstNodeType::Unknown.index() + 1,
            AST_NODE_TYPE_COUNT,
            "count should match last discriminant + 1"
        );
    }
}
