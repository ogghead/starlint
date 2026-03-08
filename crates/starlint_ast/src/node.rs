//! AST node enum and per-variant record structs.
//!
//! Each node stores its own data plus [`NodeId`] references to children,
//! forming a flat indexed tree. No recursive types.

use serde::{Deserialize, Serialize};

use crate::operator::{
    AssignmentOperator, BinaryOperator, LogicalOperator, MethodDefinitionKind, PropertyKind,
    UnaryOperator, UpdateOperator, VariableDeclarationKind,
};
use crate::types::{NodeId, Span};

// ---------------------------------------------------------------------------
// AstNode — the top-level variant enum
// ---------------------------------------------------------------------------

/// A single node in the flat indexed AST.
///
/// Each variant wraps a record struct whose child fields are [`NodeId`]
/// indices into the owning [`AstTree`](crate::AstTree).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AstNode {
    // === Program ===
    /// Root program node.
    Program(ProgramNode),

    // === Statements (17) ===
    /// `{ ... }`
    BlockStatement(BlockStatementNode),
    /// `if (test) { ... } else { ... }`
    IfStatement(IfStatementNode),
    /// `switch (discriminant) { ... }`
    SwitchStatement(SwitchStatementNode),
    /// `case test: ...` / `default: ...`
    SwitchCase(SwitchCaseNode),
    /// `for (init; test; update) { ... }`
    ForStatement(ForStatementNode),
    /// `for (left in right) { ... }`
    ForInStatement(ForInStatementNode),
    /// `for (left of right) { ... }`
    ForOfStatement(ForOfStatementNode),
    /// `while (test) { ... }`
    WhileStatement(WhileStatementNode),
    /// `do { ... } while (test)`
    DoWhileStatement(DoWhileStatementNode),
    /// `try { ... } catch { ... } finally { ... }`
    TryStatement(TryStatementNode),
    /// `catch (param) { ... }`
    CatchClause(CatchClauseNode),
    /// `throw argument`
    ThrowStatement(ThrowStatementNode),
    /// `return argument`
    ReturnStatement(ReturnStatementNode),
    /// `label: body`
    LabeledStatement(LabeledStatementNode),
    /// `break label?`
    BreakStatement(BreakStatementNode),
    /// `continue label?`
    ContinueStatement(ContinueStatementNode),
    /// `;`
    EmptyStatement(EmptyStatementNode),
    /// `with (object) { ... }`
    WithStatement(WithStatementNode),
    /// Expression used as a statement.
    ExpressionStatement(ExpressionStatementNode),

    // === Declarations (5) ===
    /// `var/let/const ...`
    VariableDeclaration(VariableDeclarationNode),
    /// Single declarator inside a `VariableDeclaration`.
    VariableDeclarator(VariableDeclaratorNode),
    /// `function name(...) { ... }`
    Function(FunctionNode),
    /// Function body (block of statements + directives).
    FunctionBody(FunctionBodyNode),
    /// `class Name { ... }`
    Class(ClassNode),
    /// `static { ... }`
    StaticBlock(StaticBlockNode),

    // === Expressions (28) ===
    /// `callee(arguments)`
    CallExpression(CallExpressionNode),
    /// `new callee(arguments)`
    NewExpression(NewExpressionNode),
    /// `left op right` (arithmetic, comparison, bitwise, etc.)
    BinaryExpression(BinaryExpressionNode),
    /// `left op right` (logical: `||`, `&&`, `??`)
    LogicalExpression(LogicalExpressionNode),
    /// `left = right` (and compound assignments)
    AssignmentExpression(AssignmentExpressionNode),
    /// `op argument` or `argument op` (unary: `!`, `typeof`, etc.)
    UnaryExpression(UnaryExpressionNode),
    /// `++x` / `x++` / `--x` / `x--`
    UpdateExpression(UpdateExpressionNode),
    /// `test ? consequent : alternate`
    ConditionalExpression(ConditionalExpressionNode),
    /// `a, b, c`
    SequenceExpression(SequenceExpressionNode),
    /// Identifier reference (variable read).
    IdentifierReference(IdentifierReferenceNode),
    /// Binding identifier (variable declaration name).
    BindingIdentifier(BindingIdentifierNode),
    /// `"string"` or `'string'`
    StringLiteral(StringLiteralNode),
    /// `42` or `3.14`
    NumericLiteral(NumericLiteralNode),
    /// `true` or `false`
    BooleanLiteral(BooleanLiteralNode),
    /// `null`
    NullLiteral(NullLiteralNode),
    /// `/pattern/flags`
    RegExpLiteral(RegExpLiteralNode),
    /// Template literal (`` `hello ${name}` ``).
    TemplateLiteral(TemplateLiteralNode),
    /// Tagged template (`` tag`hello` ``).
    TaggedTemplateExpression(TaggedTemplateExpressionNode),
    /// `[a, b, c]`
    ArrayExpression(ArrayExpressionNode),
    /// `{ a: 1, b: 2 }`
    ObjectExpression(ObjectExpressionNode),
    /// Single property in an object literal.
    ObjectProperty(ObjectPropertyNode),
    /// `...expr`
    SpreadElement(SpreadElementNode),
    /// `(params) => body`
    ArrowFunctionExpression(ArrowFunctionExpressionNode),
    /// `await expr`
    AwaitExpression(AwaitExpressionNode),
    /// `obj.prop`
    StaticMemberExpression(StaticMemberExpressionNode),
    /// `obj[expr]`
    ComputedMemberExpression(ComputedMemberExpressionNode),
    /// `obj?.prop?.method()`
    ChainExpression(ChainExpressionNode),
    /// `this`
    ThisExpression(ThisExpressionNode),
    /// `debugger;`
    DebuggerStatement(DebuggerStatementNode),

    // === Patterns (3) ===
    /// `[a, b] = ...`
    ArrayPattern(ArrayPatternNode),
    /// `{ a, b } = ...`
    ObjectPattern(ObjectPatternNode),
    /// `param = defaultValue` (default parameter or destructuring default).
    AssignmentPattern(AssignmentPatternNode),

    // === Modules (5) ===
    /// `import ... from "source"`
    ImportDeclaration(ImportDeclarationNode),
    /// Single import specifier (`{ name as local }`).
    ImportSpecifier(ImportSpecifierNode),
    /// `export { ... }`
    ExportNamedDeclaration(ExportNamedDeclarationNode),
    /// `export default ...`
    ExportDefaultDeclaration(ExportDefaultDeclarationNode),
    /// `export * from "source"`
    ExportAllDeclaration(ExportAllDeclarationNode),
    /// Single export specifier.
    ExportSpecifier(ExportSpecifierNode),

    // === Class members (2) ===
    /// Method in a class body.
    MethodDefinition(MethodDefinitionNode),
    /// Property (field) in a class body.
    PropertyDefinition(PropertyDefinitionNode),

    // === JSX (9) ===
    /// `<Component ...>...</Component>`
    JSXElement(JSXElementNode),
    /// `<Component ...>`
    JSXOpeningElement(JSXOpeningElementNode),
    /// `<>...</>`
    JSXFragment(JSXFragmentNode),
    /// `name="value"` or `name={expr}`
    JSXAttribute(JSXAttributeNode),
    /// `{...expr}`
    JSXSpreadAttribute(JSXSpreadAttributeNode),
    /// `{expression}`
    JSXExpressionContainer(JSXExpressionContainerNode),
    /// `ns:name`
    JSXNamespacedName(JSXNamespacedNameNode),
    /// Literal text in JSX.
    JSXText(JSXTextNode),

    // === TypeScript (12) ===
    /// `type Name = ...`
    TSTypeAliasDeclaration(TSTypeAliasDeclarationNode),
    /// `interface Name { ... }`
    TSInterfaceDeclaration(TSInterfaceDeclarationNode),
    /// `enum Name { ... }`
    TSEnumDeclaration(TSEnumDeclarationNode),
    /// Single enum member.
    TSEnumMember(TSEnumMemberNode),
    /// `namespace/module Name { ... }`
    TSModuleDeclaration(TSModuleDeclarationNode),
    /// `expr as Type`
    TSAsExpression(TSAsExpressionNode),
    /// `<Type>expr`
    TSTypeAssertion(TSTypeAssertionNode),
    /// `expr!`
    TSNonNullExpression(TSNonNullExpressionNode),
    /// `{ ... }` type literal.
    TSTypeLiteral(TSTypeLiteralNode),
    /// Type reference (`Foo`, `Array<T>`).
    TSTypeReference(TSTypeReferenceNode),
    /// `<T>` type parameter.
    TSTypeParameter(TSTypeParameterNode),
    /// `any`
    TSAnyKeyword(TSAnyKeywordNode),
    /// `void`
    TSVoidKeyword(TSVoidKeywordNode),

    /// Placeholder for node types not yet mapped from oxc.
    Unknown(UnknownNode),
}

impl AstNode {
    /// Return the span of this node regardless of variant.
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::Program(n) => n.span,
            Self::BlockStatement(n) => n.span,
            Self::IfStatement(n) => n.span,
            Self::SwitchStatement(n) => n.span,
            Self::SwitchCase(n) => n.span,
            Self::ForStatement(n) => n.span,
            Self::ForInStatement(n) => n.span,
            Self::ForOfStatement(n) => n.span,
            Self::WhileStatement(n) => n.span,
            Self::DoWhileStatement(n) => n.span,
            Self::TryStatement(n) => n.span,
            Self::CatchClause(n) => n.span,
            Self::ThrowStatement(n) => n.span,
            Self::ReturnStatement(n) => n.span,
            Self::LabeledStatement(n) => n.span,
            Self::BreakStatement(n) => n.span,
            Self::ContinueStatement(n) => n.span,
            Self::EmptyStatement(n) => n.span,
            Self::WithStatement(n) => n.span,
            Self::ExpressionStatement(n) => n.span,
            Self::VariableDeclaration(n) => n.span,
            Self::VariableDeclarator(n) => n.span,
            Self::Function(n) => n.span,
            Self::FunctionBody(n) => n.span,
            Self::Class(n) => n.span,
            Self::StaticBlock(n) => n.span,
            Self::CallExpression(n) => n.span,
            Self::NewExpression(n) => n.span,
            Self::BinaryExpression(n) => n.span,
            Self::LogicalExpression(n) => n.span,
            Self::AssignmentExpression(n) => n.span,
            Self::UnaryExpression(n) => n.span,
            Self::UpdateExpression(n) => n.span,
            Self::ConditionalExpression(n) => n.span,
            Self::SequenceExpression(n) => n.span,
            Self::IdentifierReference(n) => n.span,
            Self::BindingIdentifier(n) => n.span,
            Self::StringLiteral(n) => n.span,
            Self::NumericLiteral(n) => n.span,
            Self::BooleanLiteral(n) => n.span,
            Self::NullLiteral(n) => n.span,
            Self::RegExpLiteral(n) => n.span,
            Self::TemplateLiteral(n) => n.span,
            Self::TaggedTemplateExpression(n) => n.span,
            Self::ArrayExpression(n) => n.span,
            Self::ObjectExpression(n) => n.span,
            Self::ObjectProperty(n) => n.span,
            Self::SpreadElement(n) => n.span,
            Self::ArrowFunctionExpression(n) => n.span,
            Self::AwaitExpression(n) => n.span,
            Self::StaticMemberExpression(n) => n.span,
            Self::ComputedMemberExpression(n) => n.span,
            Self::ChainExpression(n) => n.span,
            Self::ThisExpression(n) => n.span,
            Self::DebuggerStatement(n) => n.span,
            Self::ArrayPattern(n) => n.span,
            Self::ObjectPattern(n) => n.span,
            Self::AssignmentPattern(n) => n.span,
            Self::ImportDeclaration(n) => n.span,
            Self::ImportSpecifier(n) => n.span,
            Self::ExportNamedDeclaration(n) => n.span,
            Self::ExportDefaultDeclaration(n) => n.span,
            Self::ExportAllDeclaration(n) => n.span,
            Self::ExportSpecifier(n) => n.span,
            Self::MethodDefinition(n) => n.span,
            Self::PropertyDefinition(n) => n.span,
            Self::JSXElement(n) => n.span,
            Self::JSXOpeningElement(n) => n.span,
            Self::JSXFragment(n) => n.span,
            Self::JSXAttribute(n) => n.span,
            Self::JSXSpreadAttribute(n) => n.span,
            Self::JSXExpressionContainer(n) => n.span,
            Self::JSXNamespacedName(n) => n.span,
            Self::JSXText(n) => n.span,
            Self::TSTypeAliasDeclaration(n) => n.span,
            Self::TSInterfaceDeclaration(n) => n.span,
            Self::TSEnumDeclaration(n) => n.span,
            Self::TSEnumMember(n) => n.span,
            Self::TSModuleDeclaration(n) => n.span,
            Self::TSAsExpression(n) => n.span,
            Self::TSTypeAssertion(n) => n.span,
            Self::TSNonNullExpression(n) => n.span,
            Self::TSTypeLiteral(n) => n.span,
            Self::TSTypeReference(n) => n.span,
            Self::TSTypeParameter(n) => n.span,
            Self::TSAnyKeyword(n) => n.span,
            Self::TSVoidKeyword(n) => n.span,
            Self::Unknown(n) => n.span,
        }
    }

    // -- Type-narrowing helpers -----------------------------------------------

    /// Narrow to `ProgramNode`.
    #[must_use]
    pub const fn as_program(&self) -> Option<&ProgramNode> {
        if let Self::Program(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `BlockStatementNode`.
    #[must_use]
    pub const fn as_block_statement(&self) -> Option<&BlockStatementNode> {
        if let Self::BlockStatement(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `IfStatementNode`.
    #[must_use]
    pub const fn as_if_statement(&self) -> Option<&IfStatementNode> {
        if let Self::IfStatement(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `SwitchStatementNode`.
    #[must_use]
    pub const fn as_switch_statement(&self) -> Option<&SwitchStatementNode> {
        if let Self::SwitchStatement(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `SwitchCaseNode`.
    #[must_use]
    pub const fn as_switch_case(&self) -> Option<&SwitchCaseNode> {
        if let Self::SwitchCase(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `ForStatementNode`.
    #[must_use]
    pub const fn as_for_statement(&self) -> Option<&ForStatementNode> {
        if let Self::ForStatement(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `ForInStatementNode`.
    #[must_use]
    pub const fn as_for_in_statement(&self) -> Option<&ForInStatementNode> {
        if let Self::ForInStatement(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `ForOfStatementNode`.
    #[must_use]
    pub const fn as_for_of_statement(&self) -> Option<&ForOfStatementNode> {
        if let Self::ForOfStatement(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `WhileStatementNode`.
    #[must_use]
    pub const fn as_while_statement(&self) -> Option<&WhileStatementNode> {
        if let Self::WhileStatement(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `DoWhileStatementNode`.
    #[must_use]
    pub const fn as_do_while_statement(&self) -> Option<&DoWhileStatementNode> {
        if let Self::DoWhileStatement(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `TryStatementNode`.
    #[must_use]
    pub const fn as_try_statement(&self) -> Option<&TryStatementNode> {
        if let Self::TryStatement(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `CatchClauseNode`.
    #[must_use]
    pub const fn as_catch_clause(&self) -> Option<&CatchClauseNode> {
        if let Self::CatchClause(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `ThrowStatementNode`.
    #[must_use]
    pub const fn as_throw_statement(&self) -> Option<&ThrowStatementNode> {
        if let Self::ThrowStatement(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `ReturnStatementNode`.
    #[must_use]
    pub const fn as_return_statement(&self) -> Option<&ReturnStatementNode> {
        if let Self::ReturnStatement(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `LabeledStatementNode`.
    #[must_use]
    pub const fn as_labeled_statement(&self) -> Option<&LabeledStatementNode> {
        if let Self::LabeledStatement(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `BreakStatementNode`.
    #[must_use]
    pub const fn as_break_statement(&self) -> Option<&BreakStatementNode> {
        if let Self::BreakStatement(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `ContinueStatementNode`.
    #[must_use]
    pub const fn as_continue_statement(&self) -> Option<&ContinueStatementNode> {
        if let Self::ContinueStatement(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `EmptyStatementNode`.
    #[must_use]
    pub const fn as_empty_statement(&self) -> Option<&EmptyStatementNode> {
        if let Self::EmptyStatement(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `WithStatementNode`.
    #[must_use]
    pub const fn as_with_statement(&self) -> Option<&WithStatementNode> {
        if let Self::WithStatement(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `ExpressionStatementNode`.
    #[must_use]
    pub const fn as_expression_statement(&self) -> Option<&ExpressionStatementNode> {
        if let Self::ExpressionStatement(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `VariableDeclarationNode`.
    #[must_use]
    pub const fn as_variable_declaration(&self) -> Option<&VariableDeclarationNode> {
        if let Self::VariableDeclaration(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `VariableDeclaratorNode`.
    #[must_use]
    pub const fn as_variable_declarator(&self) -> Option<&VariableDeclaratorNode> {
        if let Self::VariableDeclarator(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `FunctionNode`.
    #[must_use]
    pub const fn as_function(&self) -> Option<&FunctionNode> {
        if let Self::Function(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `FunctionBodyNode`.
    #[must_use]
    pub const fn as_function_body(&self) -> Option<&FunctionBodyNode> {
        if let Self::FunctionBody(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `ClassNode`.
    #[must_use]
    pub const fn as_class(&self) -> Option<&ClassNode> {
        if let Self::Class(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `StaticBlockNode`.
    #[must_use]
    pub const fn as_static_block(&self) -> Option<&StaticBlockNode> {
        if let Self::StaticBlock(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `CallExpressionNode`.
    #[must_use]
    pub const fn as_call_expression(&self) -> Option<&CallExpressionNode> {
        if let Self::CallExpression(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `NewExpressionNode`.
    #[must_use]
    pub const fn as_new_expression(&self) -> Option<&NewExpressionNode> {
        if let Self::NewExpression(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `BinaryExpressionNode`.
    #[must_use]
    pub const fn as_binary_expression(&self) -> Option<&BinaryExpressionNode> {
        if let Self::BinaryExpression(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `LogicalExpressionNode`.
    #[must_use]
    pub const fn as_logical_expression(&self) -> Option<&LogicalExpressionNode> {
        if let Self::LogicalExpression(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `AssignmentExpressionNode`.
    #[must_use]
    pub const fn as_assignment_expression(&self) -> Option<&AssignmentExpressionNode> {
        if let Self::AssignmentExpression(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `UnaryExpressionNode`.
    #[must_use]
    pub const fn as_unary_expression(&self) -> Option<&UnaryExpressionNode> {
        if let Self::UnaryExpression(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `UpdateExpressionNode`.
    #[must_use]
    pub const fn as_update_expression(&self) -> Option<&UpdateExpressionNode> {
        if let Self::UpdateExpression(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `ConditionalExpressionNode`.
    #[must_use]
    pub const fn as_conditional_expression(&self) -> Option<&ConditionalExpressionNode> {
        if let Self::ConditionalExpression(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `SequenceExpressionNode`.
    #[must_use]
    pub const fn as_sequence_expression(&self) -> Option<&SequenceExpressionNode> {
        if let Self::SequenceExpression(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `IdentifierReferenceNode`.
    #[must_use]
    pub const fn as_identifier_reference(&self) -> Option<&IdentifierReferenceNode> {
        if let Self::IdentifierReference(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `BindingIdentifierNode`.
    #[must_use]
    pub const fn as_binding_identifier(&self) -> Option<&BindingIdentifierNode> {
        if let Self::BindingIdentifier(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `StringLiteralNode`.
    #[must_use]
    pub const fn as_string_literal(&self) -> Option<&StringLiteralNode> {
        if let Self::StringLiteral(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `NumericLiteralNode`.
    #[must_use]
    pub const fn as_numeric_literal(&self) -> Option<&NumericLiteralNode> {
        if let Self::NumericLiteral(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `BooleanLiteralNode`.
    #[must_use]
    pub const fn as_boolean_literal(&self) -> Option<&BooleanLiteralNode> {
        if let Self::BooleanLiteral(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `NullLiteralNode`.
    #[must_use]
    pub const fn as_null_literal(&self) -> Option<&NullLiteralNode> {
        if let Self::NullLiteral(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `RegExpLiteralNode`.
    #[must_use]
    pub const fn as_reg_exp_literal(&self) -> Option<&RegExpLiteralNode> {
        if let Self::RegExpLiteral(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `TemplateLiteralNode`.
    #[must_use]
    pub const fn as_template_literal(&self) -> Option<&TemplateLiteralNode> {
        if let Self::TemplateLiteral(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `TaggedTemplateExpressionNode`.
    #[must_use]
    pub const fn as_tagged_template_expression(&self) -> Option<&TaggedTemplateExpressionNode> {
        if let Self::TaggedTemplateExpression(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `ArrayExpressionNode`.
    #[must_use]
    pub const fn as_array_expression(&self) -> Option<&ArrayExpressionNode> {
        if let Self::ArrayExpression(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `ObjectExpressionNode`.
    #[must_use]
    pub const fn as_object_expression(&self) -> Option<&ObjectExpressionNode> {
        if let Self::ObjectExpression(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `ObjectPropertyNode`.
    #[must_use]
    pub const fn as_object_property(&self) -> Option<&ObjectPropertyNode> {
        if let Self::ObjectProperty(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `SpreadElementNode`.
    #[must_use]
    pub const fn as_spread_element(&self) -> Option<&SpreadElementNode> {
        if let Self::SpreadElement(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `ArrowFunctionExpressionNode`.
    #[must_use]
    pub const fn as_arrow_function_expression(&self) -> Option<&ArrowFunctionExpressionNode> {
        if let Self::ArrowFunctionExpression(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `AwaitExpressionNode`.
    #[must_use]
    pub const fn as_await_expression(&self) -> Option<&AwaitExpressionNode> {
        if let Self::AwaitExpression(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `StaticMemberExpressionNode`.
    #[must_use]
    pub const fn as_static_member_expression(&self) -> Option<&StaticMemberExpressionNode> {
        if let Self::StaticMemberExpression(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `ComputedMemberExpressionNode`.
    #[must_use]
    pub const fn as_computed_member_expression(&self) -> Option<&ComputedMemberExpressionNode> {
        if let Self::ComputedMemberExpression(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `ChainExpressionNode`.
    #[must_use]
    pub const fn as_chain_expression(&self) -> Option<&ChainExpressionNode> {
        if let Self::ChainExpression(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `ThisExpressionNode`.
    #[must_use]
    pub const fn as_this_expression(&self) -> Option<&ThisExpressionNode> {
        if let Self::ThisExpression(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `DebuggerStatementNode`.
    #[must_use]
    pub const fn as_debugger_statement(&self) -> Option<&DebuggerStatementNode> {
        if let Self::DebuggerStatement(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `ArrayPatternNode`.
    #[must_use]
    pub const fn as_array_pattern(&self) -> Option<&ArrayPatternNode> {
        if let Self::ArrayPattern(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `ObjectPatternNode`.
    #[must_use]
    pub const fn as_object_pattern(&self) -> Option<&ObjectPatternNode> {
        if let Self::ObjectPattern(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `AssignmentPatternNode`.
    #[must_use]
    pub const fn as_assignment_pattern(&self) -> Option<&AssignmentPatternNode> {
        if let Self::AssignmentPattern(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `ImportDeclarationNode`.
    #[must_use]
    pub const fn as_import_declaration(&self) -> Option<&ImportDeclarationNode> {
        if let Self::ImportDeclaration(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `ImportSpecifierNode`.
    #[must_use]
    pub const fn as_import_specifier(&self) -> Option<&ImportSpecifierNode> {
        if let Self::ImportSpecifier(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `ExportNamedDeclarationNode`.
    #[must_use]
    pub const fn as_export_named_declaration(&self) -> Option<&ExportNamedDeclarationNode> {
        if let Self::ExportNamedDeclaration(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `ExportDefaultDeclarationNode`.
    #[must_use]
    pub const fn as_export_default_declaration(&self) -> Option<&ExportDefaultDeclarationNode> {
        if let Self::ExportDefaultDeclaration(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `ExportAllDeclarationNode`.
    #[must_use]
    pub const fn as_export_all_declaration(&self) -> Option<&ExportAllDeclarationNode> {
        if let Self::ExportAllDeclaration(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `ExportSpecifierNode`.
    #[must_use]
    pub const fn as_export_specifier(&self) -> Option<&ExportSpecifierNode> {
        if let Self::ExportSpecifier(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `MethodDefinitionNode`.
    #[must_use]
    pub const fn as_method_definition(&self) -> Option<&MethodDefinitionNode> {
        if let Self::MethodDefinition(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `PropertyDefinitionNode`.
    #[must_use]
    pub const fn as_property_definition(&self) -> Option<&PropertyDefinitionNode> {
        if let Self::PropertyDefinition(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `JSXElementNode`.
    #[must_use]
    pub const fn as_jsx_element(&self) -> Option<&JSXElementNode> {
        if let Self::JSXElement(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `JSXOpeningElementNode`.
    #[must_use]
    pub const fn as_jsx_opening_element(&self) -> Option<&JSXOpeningElementNode> {
        if let Self::JSXOpeningElement(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `JSXFragmentNode`.
    #[must_use]
    pub const fn as_jsx_fragment(&self) -> Option<&JSXFragmentNode> {
        if let Self::JSXFragment(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `JSXAttributeNode`.
    #[must_use]
    pub const fn as_jsx_attribute(&self) -> Option<&JSXAttributeNode> {
        if let Self::JSXAttribute(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `JSXSpreadAttributeNode`.
    #[must_use]
    pub const fn as_jsx_spread_attribute(&self) -> Option<&JSXSpreadAttributeNode> {
        if let Self::JSXSpreadAttribute(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `JSXExpressionContainerNode`.
    #[must_use]
    pub const fn as_jsx_expression_container(&self) -> Option<&JSXExpressionContainerNode> {
        if let Self::JSXExpressionContainer(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `JSXNamespacedNameNode`.
    #[must_use]
    pub const fn as_jsx_namespaced_name(&self) -> Option<&JSXNamespacedNameNode> {
        if let Self::JSXNamespacedName(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `JSXTextNode`.
    #[must_use]
    pub const fn as_jsx_text(&self) -> Option<&JSXTextNode> {
        if let Self::JSXText(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `TSTypeAliasDeclarationNode`.
    #[must_use]
    pub const fn as_ts_type_alias_declaration(&self) -> Option<&TSTypeAliasDeclarationNode> {
        if let Self::TSTypeAliasDeclaration(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `TSInterfaceDeclarationNode`.
    #[must_use]
    pub const fn as_ts_interface_declaration(&self) -> Option<&TSInterfaceDeclarationNode> {
        if let Self::TSInterfaceDeclaration(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `TSEnumDeclarationNode`.
    #[must_use]
    pub const fn as_ts_enum_declaration(&self) -> Option<&TSEnumDeclarationNode> {
        if let Self::TSEnumDeclaration(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `TSEnumMemberNode`.
    #[must_use]
    pub const fn as_ts_enum_member(&self) -> Option<&TSEnumMemberNode> {
        if let Self::TSEnumMember(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `TSModuleDeclarationNode`.
    #[must_use]
    pub const fn as_ts_module_declaration(&self) -> Option<&TSModuleDeclarationNode> {
        if let Self::TSModuleDeclaration(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `TSAsExpressionNode`.
    #[must_use]
    pub const fn as_ts_as_expression(&self) -> Option<&TSAsExpressionNode> {
        if let Self::TSAsExpression(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `TSTypeAssertionNode`.
    #[must_use]
    pub const fn as_ts_type_assertion(&self) -> Option<&TSTypeAssertionNode> {
        if let Self::TSTypeAssertion(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `TSNonNullExpressionNode`.
    #[must_use]
    pub const fn as_ts_non_null_expression(&self) -> Option<&TSNonNullExpressionNode> {
        if let Self::TSNonNullExpression(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `TSTypeLiteralNode`.
    #[must_use]
    pub const fn as_ts_type_literal(&self) -> Option<&TSTypeLiteralNode> {
        if let Self::TSTypeLiteral(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `TSTypeReferenceNode`.
    #[must_use]
    pub const fn as_ts_type_reference(&self) -> Option<&TSTypeReferenceNode> {
        if let Self::TSTypeReference(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `TSTypeParameterNode`.
    #[must_use]
    pub const fn as_ts_type_parameter(&self) -> Option<&TSTypeParameterNode> {
        if let Self::TSTypeParameter(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `TSAnyKeywordNode`.
    #[must_use]
    pub const fn as_ts_any_keyword(&self) -> Option<&TSAnyKeywordNode> {
        if let Self::TSAnyKeyword(n) = self {
            Some(n)
        } else {
            None
        }
    }
    /// Narrow to `TSVoidKeywordNode`.
    #[must_use]
    pub const fn as_ts_void_keyword(&self) -> Option<&TSVoidKeywordNode> {
        if let Self::TSVoidKeyword(n) = self {
            Some(n)
        } else {
            None
        }
    }

    // -- Category query methods -----------------------------------------------

    /// Is this node an expression?
    #[must_use]
    pub const fn is_expression(&self) -> bool {
        matches!(
            self,
            Self::CallExpression(_)
                | Self::NewExpression(_)
                | Self::BinaryExpression(_)
                | Self::LogicalExpression(_)
                | Self::AssignmentExpression(_)
                | Self::UnaryExpression(_)
                | Self::UpdateExpression(_)
                | Self::ConditionalExpression(_)
                | Self::SequenceExpression(_)
                | Self::IdentifierReference(_)
                | Self::StringLiteral(_)
                | Self::NumericLiteral(_)
                | Self::BooleanLiteral(_)
                | Self::NullLiteral(_)
                | Self::RegExpLiteral(_)
                | Self::TemplateLiteral(_)
                | Self::TaggedTemplateExpression(_)
                | Self::ArrayExpression(_)
                | Self::ObjectExpression(_)
                | Self::ArrowFunctionExpression(_)
                | Self::AwaitExpression(_)
                | Self::StaticMemberExpression(_)
                | Self::ComputedMemberExpression(_)
                | Self::ChainExpression(_)
                | Self::ThisExpression(_)
                | Self::TSAsExpression(_)
                | Self::TSTypeAssertion(_)
                | Self::TSNonNullExpression(_)
        )
    }

    /// Is this node a statement?
    #[must_use]
    pub const fn is_statement(&self) -> bool {
        matches!(
            self,
            Self::BlockStatement(_)
                | Self::IfStatement(_)
                | Self::SwitchStatement(_)
                | Self::ForStatement(_)
                | Self::ForInStatement(_)
                | Self::ForOfStatement(_)
                | Self::WhileStatement(_)
                | Self::DoWhileStatement(_)
                | Self::TryStatement(_)
                | Self::ThrowStatement(_)
                | Self::ReturnStatement(_)
                | Self::LabeledStatement(_)
                | Self::BreakStatement(_)
                | Self::ContinueStatement(_)
                | Self::EmptyStatement(_)
                | Self::WithStatement(_)
                | Self::ExpressionStatement(_)
                | Self::DebuggerStatement(_)
        )
    }

    /// Is this node a literal?
    #[must_use]
    pub const fn is_literal(&self) -> bool {
        matches!(
            self,
            Self::StringLiteral(_)
                | Self::NumericLiteral(_)
                | Self::BooleanLiteral(_)
                | Self::NullLiteral(_)
                | Self::RegExpLiteral(_)
                | Self::TemplateLiteral(_)
        )
    }

    /// Is this node a declaration?
    #[must_use]
    pub const fn is_declaration(&self) -> bool {
        matches!(
            self,
            Self::VariableDeclaration(_)
                | Self::Function(_)
                | Self::Class(_)
                | Self::ImportDeclaration(_)
                | Self::ExportNamedDeclaration(_)
                | Self::ExportDefaultDeclaration(_)
                | Self::ExportAllDeclaration(_)
                | Self::TSTypeAliasDeclaration(_)
                | Self::TSInterfaceDeclaration(_)
                | Self::TSEnumDeclaration(_)
                | Self::TSModuleDeclaration(_)
        )
    }
}

// ---------------------------------------------------------------------------
// Node record structs — Program
// ---------------------------------------------------------------------------

/// Root program node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramNode {
    /// Span covering the entire source.
    pub span: Span,
    /// Whether this is a module (`true`) or script (`false`).
    pub is_module: bool,
    /// Top-level statements / declarations.
    pub body: Box<[NodeId]>,
}

// ---------------------------------------------------------------------------
// Statements
// ---------------------------------------------------------------------------

/// Block statement (`{ ... }`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockStatementNode {
    /// Span.
    pub span: Span,
    /// Statements inside the block.
    pub body: Box<[NodeId]>,
}

/// If statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfStatementNode {
    /// Span.
    pub span: Span,
    /// Test expression.
    pub test: NodeId,
    /// Consequent statement.
    pub consequent: NodeId,
    /// Optional alternate (else) statement.
    pub alternate: Option<NodeId>,
}

/// Switch statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchStatementNode {
    /// Span.
    pub span: Span,
    /// Discriminant expression.
    pub discriminant: NodeId,
    /// Cases.
    pub cases: Box<[NodeId]>,
}

/// Single switch case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchCaseNode {
    /// Span.
    pub span: Span,
    /// Test expression (`None` for `default:`).
    pub test: Option<NodeId>,
    /// Consequent statements.
    pub consequent: Box<[NodeId]>,
}

/// For statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForStatementNode {
    /// Span.
    pub span: Span,
    /// Initializer.
    pub init: Option<NodeId>,
    /// Test condition.
    pub test: Option<NodeId>,
    /// Update expression.
    pub update: Option<NodeId>,
    /// Loop body.
    pub body: NodeId,
}

/// For-in statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForInStatementNode {
    /// Span.
    pub span: Span,
    /// Left-hand side.
    pub left: NodeId,
    /// Object to iterate over.
    pub right: NodeId,
    /// Loop body.
    pub body: NodeId,
}

/// For-of statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForOfStatementNode {
    /// Span.
    pub span: Span,
    /// Whether this is `for await...of`.
    pub is_await: bool,
    /// Left-hand side.
    pub left: NodeId,
    /// Iterable.
    pub right: NodeId,
    /// Loop body.
    pub body: NodeId,
}

/// While statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhileStatementNode {
    /// Span.
    pub span: Span,
    /// Test condition.
    pub test: NodeId,
    /// Loop body.
    pub body: NodeId,
}

/// Do-while statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoWhileStatementNode {
    /// Span.
    pub span: Span,
    /// Loop body.
    pub body: NodeId,
    /// Test condition.
    pub test: NodeId,
}

/// Try statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TryStatementNode {
    /// Span.
    pub span: Span,
    /// Try block.
    pub block: NodeId,
    /// Optional catch clause.
    pub handler: Option<NodeId>,
    /// Optional finally block.
    pub finalizer: Option<NodeId>,
}

/// Catch clause.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatchClauseNode {
    /// Span.
    pub span: Span,
    /// Optional binding pattern for the caught value.
    pub param: Option<NodeId>,
    /// Catch body.
    pub body: NodeId,
}

/// Throw statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThrowStatementNode {
    /// Span.
    pub span: Span,
    /// Argument expression.
    pub argument: NodeId,
}

/// Return statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnStatementNode {
    /// Span.
    pub span: Span,
    /// Optional return value.
    pub argument: Option<NodeId>,
}

/// Labeled statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabeledStatementNode {
    /// Span.
    pub span: Span,
    /// Label name.
    pub label: String,
    /// Labeled body.
    pub body: NodeId,
}

/// Break statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakStatementNode {
    /// Span.
    pub span: Span,
    /// Optional label name.
    pub label: Option<String>,
}

/// Continue statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinueStatementNode {
    /// Span.
    pub span: Span,
    /// Optional label name.
    pub label: Option<String>,
}

/// Empty statement (`;`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmptyStatementNode {
    /// Span.
    pub span: Span,
}

/// With statement (`with (object) { ... }`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithStatementNode {
    /// Span.
    pub span: Span,
    /// Object expression.
    pub object: NodeId,
    /// Body.
    pub body: NodeId,
}

/// Expression statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpressionStatementNode {
    /// Span.
    pub span: Span,
    /// Expression.
    pub expression: NodeId,
}

// ---------------------------------------------------------------------------
// Declarations
// ---------------------------------------------------------------------------

/// Variable declaration (`var`/`let`/`const`/`using`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDeclarationNode {
    /// Span.
    pub span: Span,
    /// Declaration kind.
    pub kind: VariableDeclarationKind,
    /// Individual declarators.
    pub declarations: Box<[NodeId]>,
}

/// Single variable declarator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDeclaratorNode {
    /// Span.
    pub span: Span,
    /// Binding pattern (identifier or destructuring).
    pub id: NodeId,
    /// Optional TypeScript type annotation (e.g. `let x: string`).
    pub type_annotation: Option<NodeId>,
    /// Optional initializer expression.
    pub init: Option<NodeId>,
}

/// Function declaration or expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionNode {
    /// Span.
    pub span: Span,
    /// Optional function name.
    pub id: Option<NodeId>,
    /// TypeScript type parameters (e.g. `<T, U>`).
    pub type_parameters: Box<[NodeId]>,
    /// Parameters (each is a binding pattern node).
    pub params: Box<[NodeId]>,
    /// TypeScript return type annotation.
    pub return_type: Option<NodeId>,
    /// Function body.
    pub body: Option<NodeId>,
    /// Whether this is an `async` function.
    pub is_async: bool,
    /// Whether this is a generator function.
    pub is_generator: bool,
    /// Whether this is a TypeScript `declare` function.
    pub is_declare: bool,
}

/// Function body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionBodyNode {
    /// Span.
    pub span: Span,
    /// Statements.
    pub statements: Box<[NodeId]>,
}

/// Class declaration or expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassNode {
    /// Span.
    pub span: Span,
    /// Optional class name.
    pub id: Option<NodeId>,
    /// Optional superclass expression.
    pub super_class: Option<NodeId>,
    /// Class body members (methods, properties, static blocks).
    pub body: Box<[NodeId]>,
    /// Whether this is a TypeScript `declare` class.
    pub is_declare: bool,
    /// Whether this is abstract.
    pub is_abstract: bool,
}

/// Static initialization block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticBlockNode {
    /// Span.
    pub span: Span,
    /// Statements.
    pub body: Box<[NodeId]>,
}

// ---------------------------------------------------------------------------
// Expressions
// ---------------------------------------------------------------------------

/// Call expression (`callee(args)`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallExpressionNode {
    /// Span.
    pub span: Span,
    /// Callee expression.
    pub callee: NodeId,
    /// Arguments.
    pub arguments: Box<[NodeId]>,
    /// Whether this is an optional call (`callee?.()`).
    pub optional: bool,
}

/// New expression (`new callee(args)`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewExpressionNode {
    /// Span.
    pub span: Span,
    /// Constructor expression.
    pub callee: NodeId,
    /// Arguments.
    pub arguments: Box<[NodeId]>,
}

/// Binary expression (`left op right`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryExpressionNode {
    /// Span.
    pub span: Span,
    /// Operator.
    pub operator: BinaryOperator,
    /// Left operand.
    pub left: NodeId,
    /// Right operand.
    pub right: NodeId,
}

/// Logical expression (`left op right`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalExpressionNode {
    /// Span.
    pub span: Span,
    /// Operator (`||`, `&&`, `??`).
    pub operator: LogicalOperator,
    /// Left operand.
    pub left: NodeId,
    /// Right operand.
    pub right: NodeId,
}

/// Assignment expression (`left op right`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentExpressionNode {
    /// Span.
    pub span: Span,
    /// Operator (`=`, `+=`, etc.).
    pub operator: AssignmentOperator,
    /// Left-hand side.
    pub left: NodeId,
    /// Right-hand side.
    pub right: NodeId,
}

/// Unary expression (`op argument`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnaryExpressionNode {
    /// Span.
    pub span: Span,
    /// Operator.
    pub operator: UnaryOperator,
    /// Operand.
    pub argument: NodeId,
}

/// Update expression (`++x`, `x--`, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateExpressionNode {
    /// Span.
    pub span: Span,
    /// Operator.
    pub operator: UpdateOperator,
    /// Whether this is a prefix operation.
    pub prefix: bool,
    /// Operand.
    pub argument: NodeId,
}

/// Conditional (ternary) expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalExpressionNode {
    /// Span.
    pub span: Span,
    /// Test expression.
    pub test: NodeId,
    /// Value if truthy.
    pub consequent: NodeId,
    /// Value if falsy.
    pub alternate: NodeId,
}

/// Sequence expression (`a, b, c`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceExpressionNode {
    /// Span.
    pub span: Span,
    /// Expressions in order.
    pub expressions: Box<[NodeId]>,
}

/// Identifier reference (reading a variable).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifierReferenceNode {
    /// Span.
    pub span: Span,
    /// Identifier name.
    pub name: String,
}

/// Binding identifier (declaring/defining a variable name).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingIdentifierNode {
    /// Span.
    pub span: Span,
    /// Identifier name.
    pub name: String,
}

/// String literal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringLiteralNode {
    /// Span.
    pub span: Span,
    /// String value (unescaped).
    pub value: String,
}

/// Numeric literal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumericLiteralNode {
    /// Span.
    pub span: Span,
    /// Numeric value.
    pub value: f64,
    /// Raw source text (e.g. `0xFF`, `1_000`).
    pub raw: String,
}

/// Boolean literal (`true` / `false`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BooleanLiteralNode {
    /// Span.
    pub span: Span,
    /// The boolean value.
    pub value: bool,
}

/// Null literal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullLiteralNode {
    /// Span.
    pub span: Span,
}

/// Regular expression literal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegExpLiteralNode {
    /// Span.
    pub span: Span,
    /// Pattern string.
    pub pattern: String,
    /// Flags string (e.g. `"gi"`).
    pub flags: String,
}

/// Template literal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateLiteralNode {
    /// Span.
    pub span: Span,
    /// Quasi (string) parts as raw strings.
    pub quasis: Box<[String]>,
    /// Expression parts (interleaved with quasis).
    pub expressions: Box<[NodeId]>,
}

/// Tagged template expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaggedTemplateExpressionNode {
    /// Span.
    pub span: Span,
    /// Tag expression.
    pub tag: NodeId,
    /// Template literal.
    pub quasi: NodeId,
}

/// Array expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrayExpressionNode {
    /// Span.
    pub span: Span,
    /// Elements (may include spread elements).
    pub elements: Box<[NodeId]>,
}

/// Object expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectExpressionNode {
    /// Span.
    pub span: Span,
    /// Properties.
    pub properties: Box<[NodeId]>,
}

/// Object property.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectPropertyNode {
    /// Span.
    pub span: Span,
    /// Property kind (init, get, set).
    pub kind: PropertyKind,
    /// Key expression.
    pub key: NodeId,
    /// Value expression.
    pub value: NodeId,
    /// Whether this is a computed property (`[expr]: value`).
    pub computed: bool,
    /// Whether this is a shorthand property (`{ x }` ≡ `{ x: x }`).
    pub shorthand: bool,
    /// Whether this is a method shorthand (`{ foo() {} }`).
    pub method: bool,
}

/// Spread element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadElementNode {
    /// Span.
    pub span: Span,
    /// Argument expression.
    pub argument: NodeId,
}

/// Arrow function expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrowFunctionExpressionNode {
    /// Span.
    pub span: Span,
    /// Parameters.
    pub params: Box<[NodeId]>,
    /// Body (block or expression).
    pub body: NodeId,
    /// Whether this is an `async` arrow.
    pub is_async: bool,
    /// Whether the body is a single expression (no braces).
    pub expression: bool,
}

/// Await expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwaitExpressionNode {
    /// Span.
    pub span: Span,
    /// Awaited expression.
    pub argument: NodeId,
}

/// Static member expression (`obj.prop`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticMemberExpressionNode {
    /// Span.
    pub span: Span,
    /// Object expression.
    pub object: NodeId,
    /// Property name.
    pub property: String,
    /// Whether this is optional (`obj?.prop`).
    pub optional: bool,
}

/// Computed member expression (`obj[expr]`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputedMemberExpressionNode {
    /// Span.
    pub span: Span,
    /// Object expression.
    pub object: NodeId,
    /// Property expression.
    pub expression: NodeId,
    /// Whether this is optional (`obj?.[expr]`).
    pub optional: bool,
}

/// Chain expression (`a?.b?.c`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainExpressionNode {
    /// Span.
    pub span: Span,
    /// Inner expression.
    pub expression: NodeId,
}

/// `this` expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThisExpressionNode {
    /// Span.
    pub span: Span,
}

/// `debugger` statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebuggerStatementNode {
    /// Span.
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Patterns
// ---------------------------------------------------------------------------

/// Array destructuring pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrayPatternNode {
    /// Span.
    pub span: Span,
    /// Elements (some may be elided/holes represented by sentinel).
    pub elements: Box<[Option<NodeId>]>,
    /// Optional rest element.
    pub rest: Option<NodeId>,
}

/// Object destructuring pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectPatternNode {
    /// Span.
    pub span: Span,
    /// Properties.
    pub properties: Box<[NodeId]>,
    /// Optional rest element.
    pub rest: Option<NodeId>,
}

/// Assignment pattern (default value in parameter or destructuring).
///
/// Represents `left = right`, e.g. `x = 42` as a default parameter value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentPatternNode {
    /// Span covering `left = right`.
    pub span: Span,
    /// The binding (left side).
    pub left: NodeId,
    /// The default value expression (right side).
    pub right: NodeId,
}

// ---------------------------------------------------------------------------
// Modules
// ---------------------------------------------------------------------------

/// Import declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDeclarationNode {
    /// Span.
    pub span: Span,
    /// Module source string (the string value without quotes).
    pub source: String,
    /// Span of the source string literal (including quotes).
    pub source_span: Span,
    /// Import specifiers.
    pub specifiers: Box<[NodeId]>,
    /// Whether this is `import type ...`.
    pub import_kind_is_type: bool,
}

/// Import specifier (`{ name as local }`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSpecifierNode {
    /// Span.
    pub span: Span,
    /// Imported name.
    pub imported: String,
    /// Local binding name.
    pub local: String,
    /// Whether this is a type-only import.
    pub is_type: bool,
}

/// Named export declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportNamedDeclarationNode {
    /// Span.
    pub span: Span,
    /// Optional declaration being exported.
    pub declaration: Option<NodeId>,
    /// Export specifiers.
    pub specifiers: Box<[NodeId]>,
    /// Optional source module (re-export).
    pub source: Option<String>,
}

/// Default export declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportDefaultDeclarationNode {
    /// Span.
    pub span: Span,
    /// Exported expression or declaration.
    pub declaration: NodeId,
}

/// Export all declaration (`export * from "source"`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportAllDeclarationNode {
    /// Span.
    pub span: Span,
    /// Source module.
    pub source: String,
    /// Optional exported name (`export * as name`).
    pub exported: Option<String>,
}

/// Export specifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSpecifierNode {
    /// Span.
    pub span: Span,
    /// Local name being exported.
    pub local: String,
    /// Exported name (may differ from local via `as`).
    pub exported: String,
}

// ---------------------------------------------------------------------------
// Class members
// ---------------------------------------------------------------------------

/// Method definition in a class body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodDefinitionNode {
    /// Span.
    pub span: Span,
    /// Method kind.
    pub kind: MethodDefinitionKind,
    /// Method key (name).
    pub key: NodeId,
    /// Method value (function).
    pub value: NodeId,
    /// Whether this is a static method.
    pub is_static: bool,
    /// Whether this is computed (`[expr]()`).
    pub computed: bool,
    /// Whether this is an accessor.
    pub is_accessor: bool,
}

/// Property (field) definition in a class body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDefinitionNode {
    /// Span.
    pub span: Span,
    /// Property key.
    pub key: NodeId,
    /// Optional initializer.
    pub value: Option<NodeId>,
    /// Whether this is a static field.
    pub is_static: bool,
    /// Whether this is computed.
    pub computed: bool,
    /// Whether this is declared (TypeScript `declare`).
    pub is_declare: bool,
}

// ---------------------------------------------------------------------------
// JSX
// ---------------------------------------------------------------------------

/// JSX element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JSXElementNode {
    /// Span.
    pub span: Span,
    /// Opening element.
    pub opening_element: NodeId,
    /// Children.
    pub children: Box<[NodeId]>,
}

/// JSX opening element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JSXOpeningElementNode {
    /// Span.
    pub span: Span,
    /// Element name (tag or component name as string).
    pub name: String,
    /// Attributes.
    pub attributes: Box<[NodeId]>,
    /// Whether this is self-closing (`<Foo />`).
    pub self_closing: bool,
}

/// JSX fragment (`<>...</>`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JSXFragmentNode {
    /// Span.
    pub span: Span,
    /// Children.
    pub children: Box<[NodeId]>,
}

/// JSX attribute (`name="value"`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JSXAttributeNode {
    /// Span.
    pub span: Span,
    /// Attribute name.
    pub name: String,
    /// Optional value (string literal node, expression container, etc.).
    pub value: Option<NodeId>,
}

/// JSX spread attribute (`{...expr}`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JSXSpreadAttributeNode {
    /// Span.
    pub span: Span,
    /// Spread argument.
    pub argument: NodeId,
}

/// JSX expression container (`{expr}`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JSXExpressionContainerNode {
    /// Span.
    pub span: Span,
    /// Expression inside the container.
    pub expression: Option<NodeId>,
}

/// JSX namespaced name (`ns:name`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JSXNamespacedNameNode {
    /// Span.
    pub span: Span,
    /// Namespace part.
    pub namespace: String,
    /// Name part.
    pub name: String,
}

/// JSX text content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JSXTextNode {
    /// Span.
    pub span: Span,
    /// Text value.
    pub value: String,
}

// ---------------------------------------------------------------------------
// TypeScript
// ---------------------------------------------------------------------------

/// TypeScript type alias declaration (`type Foo = ...`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TSTypeAliasDeclarationNode {
    /// Span.
    pub span: Span,
    /// Type name.
    pub id: NodeId,
    /// Type parameters.
    pub type_parameters: Box<[NodeId]>,
    /// The aliased type (right-hand side of `=`).
    pub type_annotation: Option<NodeId>,
}

/// TypeScript interface declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TSInterfaceDeclarationNode {
    /// Span.
    pub span: Span,
    /// Interface name.
    pub id: NodeId,
    /// Body members.
    pub body: Box<[NodeId]>,
}

/// TypeScript enum declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TSEnumDeclarationNode {
    /// Span.
    pub span: Span,
    /// Enum name.
    pub id: NodeId,
    /// Enum members.
    pub members: Box<[NodeId]>,
    /// Whether this is a `const enum`.
    pub is_const: bool,
    /// Whether this is `declare enum`.
    pub is_declare: bool,
}

/// TypeScript enum member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TSEnumMemberNode {
    /// Span.
    pub span: Span,
    /// Member name/key.
    pub id: NodeId,
    /// Optional initializer.
    pub initializer: Option<NodeId>,
}

/// TypeScript module/namespace declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TSModuleDeclarationNode {
    /// Span.
    pub span: Span,
    /// Module name.
    pub id: NodeId,
    /// Module body.
    pub body: Option<NodeId>,
    /// Whether this is `declare`.
    pub is_declare: bool,
}

/// TypeScript `as` expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TSAsExpressionNode {
    /// Span.
    pub span: Span,
    /// Expression being cast.
    pub expression: NodeId,
}

/// TypeScript type assertion (`<Type>expr`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TSTypeAssertionNode {
    /// Span.
    pub span: Span,
    /// Expression being asserted.
    pub expression: NodeId,
}

/// TypeScript non-null assertion (`expr!`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TSNonNullExpressionNode {
    /// Span.
    pub span: Span,
    /// Expression.
    pub expression: NodeId,
}

/// TypeScript type literal (`{ ... }`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TSTypeLiteralNode {
    /// Span.
    pub span: Span,
    /// Members.
    pub members: Box<[NodeId]>,
}

/// TypeScript type reference (`Foo`, `Array<T>`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TSTypeReferenceNode {
    /// Span.
    pub span: Span,
    /// Type name.
    pub type_name: String,
    /// Type arguments.
    pub type_arguments: Box<[NodeId]>,
}

/// TypeScript type parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TSTypeParameterNode {
    /// Span.
    pub span: Span,
    /// Parameter name.
    pub name: String,
    /// Optional constraint.
    pub constraint: Option<NodeId>,
    /// Optional default type.
    pub default: Option<NodeId>,
}

/// TypeScript `any` keyword.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TSAnyKeywordNode {
    /// Span.
    pub span: Span,
}

/// TypeScript `void` keyword.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TSVoidKeywordNode {
    /// Span.
    pub span: Span,
}

/// Placeholder for node types not yet mapped from oxc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownNode {
    /// Span.
    pub span: Span,
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;
    use crate::operator::VariableDeclarationKind;
    use crate::types::{NodeId, Span};

    // -----------------------------------------------------------------------
    // Helper constructors for representative node types
    // -----------------------------------------------------------------------

    fn make_program() -> AstNode {
        AstNode::Program(ProgramNode {
            span: Span::new(0, 100),
            is_module: true,
            body: Box::new([]),
        })
    }

    fn make_if_statement() -> AstNode {
        AstNode::IfStatement(IfStatementNode {
            span: Span::new(5, 50),
            test: NodeId(1),
            consequent: NodeId(2),
            alternate: None,
        })
    }

    fn make_call_expression() -> AstNode {
        AstNode::CallExpression(CallExpressionNode {
            span: Span::new(10, 30),
            callee: NodeId(1),
            arguments: Box::new([NodeId(2), NodeId(3)]),
            optional: false,
        })
    }

    fn make_string_literal() -> AstNode {
        AstNode::StringLiteral(StringLiteralNode {
            span: Span::new(0, 7),
            value: String::from("hello"),
        })
    }

    fn make_variable_declaration() -> AstNode {
        AstNode::VariableDeclaration(VariableDeclarationNode {
            span: Span::new(0, 20),
            kind: VariableDeclarationKind::Const,
            declarations: Box::new([NodeId(1)]),
        })
    }

    fn make_function() -> AstNode {
        AstNode::Function(FunctionNode {
            span: Span::new(0, 40),
            id: Some(NodeId(1)),
            type_parameters: Box::new([]),
            params: Box::new([]),
            return_type: None,
            body: Some(NodeId(2)),
            is_async: false,
            is_generator: false,
            is_declare: false,
        })
    }

    fn make_jsx_element() -> AstNode {
        AstNode::JSXElement(JSXElementNode {
            span: Span::new(0, 25),
            opening_element: NodeId(1),
            children: Box::new([]),
        })
    }

    fn make_ts_interface_declaration() -> AstNode {
        AstNode::TSInterfaceDeclaration(TSInterfaceDeclarationNode {
            span: Span::new(0, 35),
            id: NodeId(1),
            body: Box::new([]),
        })
    }

    fn make_identifier_reference() -> AstNode {
        AstNode::IdentifierReference(IdentifierReferenceNode {
            span: Span::new(0, 3),
            name: String::from("foo"),
        })
    }

    fn make_numeric_literal() -> AstNode {
        AstNode::NumericLiteral(NumericLiteralNode {
            span: Span::new(0, 2),
            value: 42.0,
            raw: String::from("42"),
        })
    }

    fn make_boolean_literal() -> AstNode {
        AstNode::BooleanLiteral(BooleanLiteralNode {
            span: Span::new(0, 4),
            value: true,
        })
    }

    fn make_null_literal() -> AstNode {
        AstNode::NullLiteral(NullLiteralNode {
            span: Span::new(0, 4),
        })
    }

    fn make_regexp_literal() -> AstNode {
        AstNode::RegExpLiteral(RegExpLiteralNode {
            span: Span::new(0, 6),
            pattern: String::from("abc"),
            flags: String::from("g"),
        })
    }

    fn make_class() -> AstNode {
        AstNode::Class(ClassNode {
            span: Span::new(0, 30),
            id: Some(NodeId(1)),
            super_class: None,
            body: Box::new([]),
            is_declare: false,
            is_abstract: false,
        })
    }

    fn make_import_declaration() -> AstNode {
        AstNode::ImportDeclaration(ImportDeclarationNode {
            span: Span::new(0, 25),
            source: String::from("./mod"),
            source_span: Span::new(15, 22),
            specifiers: Box::new([]),
            import_kind_is_type: false,
        })
    }

    fn make_ts_enum_declaration() -> AstNode {
        AstNode::TSEnumDeclaration(TSEnumDeclarationNode {
            span: Span::new(0, 20),
            id: NodeId(1),
            members: Box::new([]),
            is_const: false,
            is_declare: false,
        })
    }

    fn make_template_literal() -> AstNode {
        AstNode::TemplateLiteral(TemplateLiteralNode {
            span: Span::new(0, 15),
            quasis: Box::new([String::from("hello "), String::new()]),
            expressions: Box::new([NodeId(1)]),
        })
    }

    fn make_debugger_statement() -> AstNode {
        AstNode::DebuggerStatement(DebuggerStatementNode {
            span: Span::new(10, 18),
        })
    }

    fn make_empty_statement() -> AstNode {
        AstNode::EmptyStatement(EmptyStatementNode {
            span: Span::new(0, 1),
        })
    }

    fn make_this_expression() -> AstNode {
        AstNode::ThisExpression(ThisExpressionNode {
            span: Span::new(0, 4),
        })
    }

    fn make_ts_as_expression() -> AstNode {
        AstNode::TSAsExpression(TSAsExpressionNode {
            span: Span::new(0, 12),
            expression: NodeId(1),
        })
    }

    fn make_export_named_declaration() -> AstNode {
        AstNode::ExportNamedDeclaration(ExportNamedDeclarationNode {
            span: Span::new(0, 30),
            declaration: None,
            specifiers: Box::new([]),
            source: None,
        })
    }

    fn make_export_default_declaration() -> AstNode {
        AstNode::ExportDefaultDeclaration(ExportDefaultDeclarationNode {
            span: Span::new(0, 25),
            declaration: NodeId(1),
        })
    }

    fn make_export_all_declaration() -> AstNode {
        AstNode::ExportAllDeclaration(ExportAllDeclarationNode {
            span: Span::new(0, 20),
            source: String::from("./mod"),
            exported: None,
        })
    }

    fn make_ts_type_alias_declaration() -> AstNode {
        AstNode::TSTypeAliasDeclaration(TSTypeAliasDeclarationNode {
            span: Span::new(0, 18),
            id: NodeId(1),
            type_parameters: Box::new([]),
            type_annotation: None,
        })
    }

    fn make_ts_module_declaration() -> AstNode {
        AstNode::TSModuleDeclaration(TSModuleDeclarationNode {
            span: Span::new(0, 22),
            id: NodeId(1),
            body: None,
            is_declare: true,
        })
    }

    // -----------------------------------------------------------------------
    // span() tests
    // -----------------------------------------------------------------------

    #[test]
    fn span_returns_correct_span_for_program() {
        let node = make_program();
        assert_eq!(
            node.span(),
            Span::new(0, 100),
            "Program span should be (0, 100)"
        );
    }

    #[test]
    fn span_returns_correct_span_for_if_statement() {
        let node = make_if_statement();
        assert_eq!(
            node.span(),
            Span::new(5, 50),
            "IfStatement span should be (5, 50)"
        );
    }

    #[test]
    fn span_returns_correct_span_for_call_expression() {
        let node = make_call_expression();
        assert_eq!(
            node.span(),
            Span::new(10, 30),
            "CallExpression span should be (10, 30)"
        );
    }

    #[test]
    fn span_returns_correct_span_for_string_literal() {
        let node = make_string_literal();
        assert_eq!(
            node.span(),
            Span::new(0, 7),
            "StringLiteral span should be (0, 7)"
        );
    }

    #[test]
    fn span_returns_correct_span_for_debugger_statement() {
        let node = make_debugger_statement();
        assert_eq!(
            node.span(),
            Span::new(10, 18),
            "DebuggerStatement span should be (10, 18)"
        );
    }

    #[test]
    fn span_returns_correct_span_for_jsx_element() {
        let node = make_jsx_element();
        assert_eq!(
            node.span(),
            Span::new(0, 25),
            "JSXElement span should be (0, 25)"
        );
    }

    #[test]
    fn span_returns_correct_span_for_ts_interface() {
        let node = make_ts_interface_declaration();
        assert_eq!(
            node.span(),
            Span::new(0, 35),
            "TSInterfaceDeclaration span should be (0, 35)"
        );
    }

    #[test]
    fn span_returns_correct_span_for_unknown() {
        let node = AstNode::Unknown(UnknownNode {
            span: Span::new(99, 200),
        });
        assert_eq!(
            node.span(),
            Span::new(99, 200),
            "Unknown span should be (99, 200)"
        );
    }

    // -----------------------------------------------------------------------
    // as_*() positive tests — verify matching getter returns Some
    // -----------------------------------------------------------------------

    #[test]
    fn as_program_returns_some_for_program_node() {
        let node = make_program();
        assert!(
            node.as_program().is_some(),
            "as_program should return Some for Program variant"
        );
    }

    #[test]
    fn as_if_statement_returns_some_for_if_statement_node() {
        let node = make_if_statement();
        assert!(
            node.as_if_statement().is_some(),
            "as_if_statement should return Some for IfStatement variant"
        );
    }

    #[test]
    fn as_call_expression_returns_some_for_call_expression_node() {
        let node = make_call_expression();
        assert!(
            node.as_call_expression().is_some(),
            "as_call_expression should return Some for CallExpression variant"
        );
    }

    #[test]
    fn as_string_literal_returns_some_for_string_literal_node() {
        let node = make_string_literal();
        assert!(
            node.as_string_literal().is_some(),
            "as_string_literal should return Some for StringLiteral variant"
        );
    }

    #[test]
    fn as_variable_declaration_returns_some_for_variable_declaration_node() {
        let node = make_variable_declaration();
        assert!(
            node.as_variable_declaration().is_some(),
            "as_variable_declaration should return Some for VariableDeclaration variant"
        );
    }

    #[test]
    fn as_function_returns_some_for_function_node() {
        let node = make_function();
        assert!(
            node.as_function().is_some(),
            "as_function should return Some for Function variant"
        );
    }

    #[test]
    fn as_jsx_element_returns_some_for_jsx_element_node() {
        let node = make_jsx_element();
        assert!(
            node.as_jsx_element().is_some(),
            "as_jsx_element should return Some for JSXElement variant"
        );
    }

    #[test]
    fn as_ts_interface_declaration_returns_some_for_ts_interface_node() {
        let node = make_ts_interface_declaration();
        assert!(
            node.as_ts_interface_declaration().is_some(),
            "as_ts_interface_declaration should return Some for TSInterfaceDeclaration variant"
        );
    }

    #[test]
    fn as_identifier_reference_returns_some_for_identifier_reference_node() {
        let node = make_identifier_reference();
        assert!(
            node.as_identifier_reference().is_some(),
            "as_identifier_reference should return Some for IdentifierReference variant"
        );
    }

    #[test]
    fn as_class_returns_some_for_class_node() {
        let node = make_class();
        assert!(
            node.as_class().is_some(),
            "as_class should return Some for Class variant"
        );
    }

    #[test]
    fn as_import_declaration_returns_some_for_import_declaration_node() {
        let node = make_import_declaration();
        assert!(
            node.as_import_declaration().is_some(),
            "as_import_declaration should return Some for ImportDeclaration variant"
        );
    }

    #[test]
    fn as_ts_enum_declaration_returns_some_for_ts_enum_node() {
        let node = make_ts_enum_declaration();
        assert!(
            node.as_ts_enum_declaration().is_some(),
            "as_ts_enum_declaration should return Some for TSEnumDeclaration variant"
        );
    }

    // -----------------------------------------------------------------------
    // as_*() positive tests — verify the returned reference has correct data
    // -----------------------------------------------------------------------

    #[test]
    fn as_program_returns_correct_inner_data() {
        let node = make_program();
        let inner = node.as_program();
        assert!(inner.is_some(), "as_program should return Some");
        if let Some(prog) = inner {
            assert!(prog.is_module, "ProgramNode.is_module should be true");
            assert!(prog.body.is_empty(), "ProgramNode.body should be empty");
        }
    }

    #[test]
    fn as_string_literal_returns_correct_value() {
        let node = make_string_literal();
        let maybe_lit = node.as_string_literal();
        assert!(
            maybe_lit.is_some(),
            "as_string_literal should return Some for StringLiteral variant"
        );
        #[allow(clippy::unwrap_used)]
        let lit = maybe_lit.unwrap();
        assert_eq!(
            lit.value, "hello",
            "StringLiteralNode.value should be 'hello'"
        );
    }

    #[test]
    fn as_call_expression_returns_correct_arguments_count() {
        let node = make_call_expression();
        let maybe_call = node.as_call_expression();
        assert!(
            maybe_call.is_some(),
            "as_call_expression should return Some for CallExpression variant"
        );
        #[allow(clippy::unwrap_used)]
        let call = maybe_call.unwrap();
        assert_eq!(
            call.arguments.len(),
            2,
            "CallExpressionNode should have 2 arguments"
        );
        assert!(
            !call.optional,
            "CallExpressionNode.optional should be false"
        );
    }

    #[test]
    fn as_identifier_reference_returns_correct_name() {
        let node = make_identifier_reference();
        let maybe_id = node.as_identifier_reference();
        assert!(
            maybe_id.is_some(),
            "as_identifier_reference should return Some for IdentifierReference variant"
        );
        #[allow(clippy::unwrap_used)]
        let ident = maybe_id.unwrap();
        assert_eq!(
            ident.name, "foo",
            "IdentifierReferenceNode.name should be 'foo'"
        );
    }

    // -----------------------------------------------------------------------
    // as_*() negative tests — mismatched getters return None
    // -----------------------------------------------------------------------

    #[test]
    fn as_getters_return_none_for_non_matching_variants() {
        let program = make_program();

        assert!(
            program.as_if_statement().is_none(),
            "as_if_statement should return None for Program variant"
        );
        assert!(
            program.as_call_expression().is_none(),
            "as_call_expression should return None for Program variant"
        );
        assert!(
            program.as_string_literal().is_none(),
            "as_string_literal should return None for Program variant"
        );
        assert!(
            program.as_variable_declaration().is_none(),
            "as_variable_declaration should return None for Program variant"
        );
        assert!(
            program.as_function().is_none(),
            "as_function should return None for Program variant"
        );
        assert!(
            program.as_jsx_element().is_none(),
            "as_jsx_element should return None for Program variant"
        );
        assert!(
            program.as_ts_interface_declaration().is_none(),
            "as_ts_interface_declaration should return None for Program variant"
        );
        assert!(
            program.as_class().is_none(),
            "as_class should return None for Program variant"
        );
        assert!(
            program.as_import_declaration().is_none(),
            "as_import_declaration should return None for Program variant"
        );
        assert!(
            program.as_identifier_reference().is_none(),
            "as_identifier_reference should return None for Program variant"
        );
    }

    #[test]
    fn as_program_returns_none_for_expression_variant() {
        let call = make_call_expression();
        assert!(
            call.as_program().is_none(),
            "as_program should return None for CallExpression variant"
        );
    }

    #[test]
    fn as_statement_getters_return_none_for_expression_variant() {
        let str_lit = make_string_literal();

        assert!(
            str_lit.as_if_statement().is_none(),
            "as_if_statement should return None for StringLiteral variant"
        );
        assert!(
            str_lit.as_block_statement().is_none(),
            "as_block_statement should return None for StringLiteral variant"
        );
        assert!(
            str_lit.as_return_statement().is_none(),
            "as_return_statement should return None for StringLiteral variant"
        );
        assert!(
            str_lit.as_debugger_statement().is_none(),
            "as_debugger_statement should return None for StringLiteral variant"
        );
    }

    #[test]
    fn as_jsx_getters_return_none_for_non_jsx_variant() {
        let func = make_function();

        assert!(
            func.as_jsx_element().is_none(),
            "as_jsx_element should return None for Function variant"
        );
        assert!(
            func.as_jsx_opening_element().is_none(),
            "as_jsx_opening_element should return None for Function variant"
        );
        assert!(
            func.as_jsx_fragment().is_none(),
            "as_jsx_fragment should return None for Function variant"
        );
        assert!(
            func.as_jsx_attribute().is_none(),
            "as_jsx_attribute should return None for Function variant"
        );
        assert!(
            func.as_jsx_text().is_none(),
            "as_jsx_text should return None for Function variant"
        );
    }

    #[test]
    fn as_ts_getters_return_none_for_non_ts_variant() {
        let if_stmt = make_if_statement();

        assert!(
            if_stmt.as_ts_type_alias_declaration().is_none(),
            "as_ts_type_alias_declaration should return None for IfStatement variant"
        );
        assert!(
            if_stmt.as_ts_interface_declaration().is_none(),
            "as_ts_interface_declaration should return None for IfStatement variant"
        );
        assert!(
            if_stmt.as_ts_enum_declaration().is_none(),
            "as_ts_enum_declaration should return None for IfStatement variant"
        );
        assert!(
            if_stmt.as_ts_as_expression().is_none(),
            "as_ts_as_expression should return None for IfStatement variant"
        );
        assert!(
            if_stmt.as_ts_non_null_expression().is_none(),
            "as_ts_non_null_expression should return None for IfStatement variant"
        );
    }

    // -----------------------------------------------------------------------
    // is_expression() tests
    // -----------------------------------------------------------------------

    #[test]
    fn is_expression_returns_true_for_call_expression() {
        assert!(
            make_call_expression().is_expression(),
            "CallExpression should be an expression"
        );
    }

    #[test]
    fn is_expression_returns_true_for_string_literal() {
        assert!(
            make_string_literal().is_expression(),
            "StringLiteral should be an expression"
        );
    }

    #[test]
    fn is_expression_returns_true_for_identifier_reference() {
        assert!(
            make_identifier_reference().is_expression(),
            "IdentifierReference should be an expression"
        );
    }

    #[test]
    fn is_expression_returns_true_for_this_expression() {
        assert!(
            make_this_expression().is_expression(),
            "ThisExpression should be an expression"
        );
    }

    #[test]
    fn is_expression_returns_true_for_ts_as_expression() {
        assert!(
            make_ts_as_expression().is_expression(),
            "TSAsExpression should be an expression"
        );
    }

    #[test]
    fn is_expression_returns_true_for_numeric_literal() {
        assert!(
            make_numeric_literal().is_expression(),
            "NumericLiteral should be an expression"
        );
    }

    #[test]
    fn is_expression_returns_true_for_boolean_literal() {
        assert!(
            make_boolean_literal().is_expression(),
            "BooleanLiteral should be an expression"
        );
    }

    #[test]
    fn is_expression_returns_true_for_null_literal() {
        assert!(
            make_null_literal().is_expression(),
            "NullLiteral should be an expression"
        );
    }

    #[test]
    fn is_expression_returns_true_for_template_literal() {
        assert!(
            make_template_literal().is_expression(),
            "TemplateLiteral should be an expression"
        );
    }

    #[test]
    fn is_expression_returns_false_for_program() {
        assert!(
            !make_program().is_expression(),
            "Program should not be an expression"
        );
    }

    #[test]
    fn is_expression_returns_false_for_if_statement() {
        assert!(
            !make_if_statement().is_expression(),
            "IfStatement should not be an expression"
        );
    }

    #[test]
    fn is_expression_returns_false_for_variable_declaration() {
        assert!(
            !make_variable_declaration().is_expression(),
            "VariableDeclaration should not be an expression"
        );
    }

    #[test]
    fn is_expression_returns_false_for_jsx_element() {
        assert!(
            !make_jsx_element().is_expression(),
            "JSXElement should not be an expression"
        );
    }

    #[test]
    fn is_expression_returns_false_for_import_declaration() {
        assert!(
            !make_import_declaration().is_expression(),
            "ImportDeclaration should not be an expression"
        );
    }

    // -----------------------------------------------------------------------
    // is_statement() tests
    // -----------------------------------------------------------------------

    #[test]
    fn is_statement_returns_true_for_if_statement() {
        assert!(
            make_if_statement().is_statement(),
            "IfStatement should be a statement"
        );
    }

    #[test]
    fn is_statement_returns_true_for_debugger_statement() {
        assert!(
            make_debugger_statement().is_statement(),
            "DebuggerStatement should be a statement"
        );
    }

    #[test]
    fn is_statement_returns_true_for_empty_statement() {
        assert!(
            make_empty_statement().is_statement(),
            "EmptyStatement should be a statement"
        );
    }

    #[test]
    fn is_statement_returns_false_for_program() {
        assert!(
            !make_program().is_statement(),
            "Program should not be a statement"
        );
    }

    #[test]
    fn is_statement_returns_false_for_call_expression() {
        assert!(
            !make_call_expression().is_statement(),
            "CallExpression should not be a statement"
        );
    }

    #[test]
    fn is_statement_returns_false_for_variable_declaration() {
        assert!(
            !make_variable_declaration().is_statement(),
            "VariableDeclaration should not be a statement"
        );
    }

    #[test]
    fn is_statement_returns_false_for_class() {
        assert!(
            !make_class().is_statement(),
            "Class should not be a statement"
        );
    }

    #[test]
    fn is_statement_returns_false_for_jsx_element() {
        assert!(
            !make_jsx_element().is_statement(),
            "JSXElement should not be a statement"
        );
    }

    // -----------------------------------------------------------------------
    // is_literal() tests
    // -----------------------------------------------------------------------

    #[test]
    fn is_literal_returns_true_for_string_literal() {
        assert!(
            make_string_literal().is_literal(),
            "StringLiteral should be a literal"
        );
    }

    #[test]
    fn is_literal_returns_true_for_numeric_literal() {
        assert!(
            make_numeric_literal().is_literal(),
            "NumericLiteral should be a literal"
        );
    }

    #[test]
    fn is_literal_returns_true_for_boolean_literal() {
        assert!(
            make_boolean_literal().is_literal(),
            "BooleanLiteral should be a literal"
        );
    }

    #[test]
    fn is_literal_returns_true_for_null_literal() {
        assert!(
            make_null_literal().is_literal(),
            "NullLiteral should be a literal"
        );
    }

    #[test]
    fn is_literal_returns_true_for_regexp_literal() {
        assert!(
            make_regexp_literal().is_literal(),
            "RegExpLiteral should be a literal"
        );
    }

    #[test]
    fn is_literal_returns_true_for_template_literal() {
        assert!(
            make_template_literal().is_literal(),
            "TemplateLiteral should be a literal"
        );
    }

    #[test]
    fn is_literal_returns_false_for_identifier_reference() {
        assert!(
            !make_identifier_reference().is_literal(),
            "IdentifierReference should not be a literal"
        );
    }

    #[test]
    fn is_literal_returns_false_for_call_expression() {
        assert!(
            !make_call_expression().is_literal(),
            "CallExpression should not be a literal"
        );
    }

    #[test]
    fn is_literal_returns_false_for_program() {
        assert!(
            !make_program().is_literal(),
            "Program should not be a literal"
        );
    }

    #[test]
    fn is_literal_returns_false_for_if_statement() {
        assert!(
            !make_if_statement().is_literal(),
            "IfStatement should not be a literal"
        );
    }

    // -----------------------------------------------------------------------
    // is_declaration() tests
    // -----------------------------------------------------------------------

    #[test]
    fn is_declaration_returns_true_for_variable_declaration() {
        assert!(
            make_variable_declaration().is_declaration(),
            "VariableDeclaration should be a declaration"
        );
    }

    #[test]
    fn is_declaration_returns_true_for_function() {
        assert!(
            make_function().is_declaration(),
            "Function should be a declaration"
        );
    }

    #[test]
    fn is_declaration_returns_true_for_class() {
        assert!(
            make_class().is_declaration(),
            "Class should be a declaration"
        );
    }

    #[test]
    fn is_declaration_returns_true_for_import_declaration() {
        assert!(
            make_import_declaration().is_declaration(),
            "ImportDeclaration should be a declaration"
        );
    }

    #[test]
    fn is_declaration_returns_true_for_export_named_declaration() {
        assert!(
            make_export_named_declaration().is_declaration(),
            "ExportNamedDeclaration should be a declaration"
        );
    }

    #[test]
    fn is_declaration_returns_true_for_export_default_declaration() {
        assert!(
            make_export_default_declaration().is_declaration(),
            "ExportDefaultDeclaration should be a declaration"
        );
    }

    #[test]
    fn is_declaration_returns_true_for_export_all_declaration() {
        assert!(
            make_export_all_declaration().is_declaration(),
            "ExportAllDeclaration should be a declaration"
        );
    }

    #[test]
    fn is_declaration_returns_true_for_ts_type_alias_declaration() {
        assert!(
            make_ts_type_alias_declaration().is_declaration(),
            "TSTypeAliasDeclaration should be a declaration"
        );
    }

    #[test]
    fn is_declaration_returns_true_for_ts_interface_declaration() {
        assert!(
            make_ts_interface_declaration().is_declaration(),
            "TSInterfaceDeclaration should be a declaration"
        );
    }

    #[test]
    fn is_declaration_returns_true_for_ts_enum_declaration() {
        assert!(
            make_ts_enum_declaration().is_declaration(),
            "TSEnumDeclaration should be a declaration"
        );
    }

    #[test]
    fn is_declaration_returns_true_for_ts_module_declaration() {
        assert!(
            make_ts_module_declaration().is_declaration(),
            "TSModuleDeclaration should be a declaration"
        );
    }

    #[test]
    fn is_declaration_returns_false_for_call_expression() {
        assert!(
            !make_call_expression().is_declaration(),
            "CallExpression should not be a declaration"
        );
    }

    #[test]
    fn is_declaration_returns_false_for_if_statement() {
        assert!(
            !make_if_statement().is_declaration(),
            "IfStatement should not be a declaration"
        );
    }

    #[test]
    fn is_declaration_returns_false_for_string_literal() {
        assert!(
            !make_string_literal().is_declaration(),
            "StringLiteral should not be a declaration"
        );
    }

    #[test]
    fn is_declaration_returns_false_for_program() {
        assert!(
            !make_program().is_declaration(),
            "Program should not be a declaration"
        );
    }

    #[test]
    fn is_declaration_returns_false_for_jsx_element() {
        assert!(
            !make_jsx_element().is_declaration(),
            "JSXElement should not be a declaration"
        );
    }

    // -----------------------------------------------------------------------
    // Category exclusivity tests — verify a node is in exactly one category
    // (or none, for nodes like Program, patterns, etc.)
    // -----------------------------------------------------------------------

    #[test]
    fn categories_are_exclusive_for_if_statement() {
        let node = make_if_statement();
        assert!(node.is_statement(), "IfStatement should be a statement");
        assert!(
            !node.is_expression(),
            "IfStatement should not be an expression"
        );
        assert!(!node.is_literal(), "IfStatement should not be a literal");
        assert!(
            !node.is_declaration(),
            "IfStatement should not be a declaration"
        );
    }

    #[test]
    fn categories_are_exclusive_for_call_expression() {
        let node = make_call_expression();
        assert!(
            node.is_expression(),
            "CallExpression should be an expression"
        );
        assert!(
            !node.is_statement(),
            "CallExpression should not be a statement"
        );
        assert!(!node.is_literal(), "CallExpression should not be a literal");
        assert!(
            !node.is_declaration(),
            "CallExpression should not be a declaration"
        );
    }

    #[test]
    fn string_literal_is_both_expression_and_literal() {
        let node = make_string_literal();
        assert!(
            node.is_expression(),
            "StringLiteral should be an expression"
        );
        assert!(node.is_literal(), "StringLiteral should be a literal");
        assert!(
            !node.is_statement(),
            "StringLiteral should not be a statement"
        );
        assert!(
            !node.is_declaration(),
            "StringLiteral should not be a declaration"
        );
    }

    #[test]
    fn program_is_in_no_category() {
        let node = make_program();
        assert!(!node.is_expression(), "Program should not be an expression");
        assert!(!node.is_statement(), "Program should not be a statement");
        assert!(!node.is_literal(), "Program should not be a literal");
        assert!(
            !node.is_declaration(),
            "Program should not be a declaration"
        );
    }

    #[test]
    fn jsx_element_is_in_no_predicate_category() {
        let node = make_jsx_element();
        assert!(
            !node.is_expression(),
            "JSXElement should not be an expression"
        );
        assert!(!node.is_statement(), "JSXElement should not be a statement");
        assert!(!node.is_literal(), "JSXElement should not be a literal");
        assert!(
            !node.is_declaration(),
            "JSXElement should not be a declaration"
        );
    }

    #[test]
    fn variable_declaration_is_declaration_only() {
        let node = make_variable_declaration();
        assert!(
            node.is_declaration(),
            "VariableDeclaration should be a declaration"
        );
        assert!(
            !node.is_expression(),
            "VariableDeclaration should not be an expression"
        );
        assert!(
            !node.is_statement(),
            "VariableDeclaration should not be a statement"
        );
        assert!(
            !node.is_literal(),
            "VariableDeclaration should not be a literal"
        );
    }

    // -----------------------------------------------------------------------
    // Serde roundtrip tests
    // -----------------------------------------------------------------------

    #[test]
    fn serde_roundtrip_debugger_statement() {
        let node = make_debugger_statement();
        let json = serde_json::to_string(&node);
        assert!(json.is_ok(), "DebuggerStatement should serialize");
        let back: Result<AstNode, _> = serde_json::from_str(json.as_deref().unwrap_or(""));
        assert!(back.is_ok(), "DebuggerStatement should deserialize");
    }

    #[test]
    fn serde_roundtrip_program_preserves_span() {
        let node = make_program();
        let json = serde_json::to_string(&node);
        assert!(json.is_ok(), "Program should serialize");
        let back: Result<AstNode, _> = serde_json::from_str(json.as_deref().unwrap_or(""));
        assert!(back.is_ok(), "Program should deserialize");
        if let Ok(deserialized) = back {
            assert_eq!(
                deserialized.span(),
                Span::new(0, 100),
                "Deserialized Program span should match original"
            );
        }
    }

    #[test]
    fn serde_roundtrip_call_expression_preserves_data() {
        let node = make_call_expression();
        let json = serde_json::to_string(&node);
        assert!(json.is_ok(), "CallExpression should serialize");
        let back: Result<AstNode, _> = serde_json::from_str(json.as_deref().unwrap_or(""));
        assert!(back.is_ok(), "CallExpression should deserialize");
        if let Ok(deserialized) = back {
            assert!(
                deserialized.as_call_expression().is_some(),
                "Deserialized node should still be CallExpression"
            );
        }
    }
}
