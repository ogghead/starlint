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

    // === Patterns (2) ===
    /// `[a, b] = ...`
    ArrayPattern(ArrayPatternNode),
    /// `{ a, b } = ...`
    ObjectPattern(ObjectPatternNode),

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
        }
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
    /// Parameters (each is a binding pattern node).
    pub params: Box<[NodeId]>,
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

// ---------------------------------------------------------------------------
// Modules
// ---------------------------------------------------------------------------

/// Import declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDeclarationNode {
    /// Span.
    pub span: Span,
    /// Module source string.
    pub source: String,
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

#[cfg(test)]
mod tests {
    use super::{AstNode, DebuggerStatementNode};
    use crate::types::Span;

    #[test]
    fn ast_node_span() {
        let node = AstNode::DebuggerStatement(DebuggerStatementNode {
            span: Span::new(10, 18),
        });
        assert_eq!(node.span(), Span::new(10, 18), "span should match");
    }

    #[test]
    fn ast_node_serde_roundtrip() {
        let node = AstNode::DebuggerStatement(DebuggerStatementNode {
            span: Span::new(0, 8),
        });
        let json = serde_json::to_string(&node);
        assert!(json.is_ok(), "should serialize");
        let back: Result<AstNode, _> = serde_json::from_str(json.as_deref().unwrap_or(""));
        assert!(back.is_ok(), "should deserialize");
    }
}
