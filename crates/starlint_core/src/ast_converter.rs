//! Converter from oxc's arena-allocated AST to [`AstTree`].
//!
//! The converter walks an oxc `Program` recursively and produces a flat
//! indexed [`AstTree`] where each node references children by [`NodeId`].
//!
//! Node types not yet represented in [`AstNode`] are mapped to
//! [`AstNode::Unknown`].

use oxc_ast::ast::{
    self as oxc, Argument, ArrayExpressionElement, AssignmentTarget, BindingPattern, ChainElement,
    ClassElement, ExportDefaultDeclarationKind, Expression, ForStatementInit, ForStatementLeft,
    ImportDeclarationSpecifier, ObjectPropertyKind, PropertyKey, SimpleAssignmentTarget, Statement,
    TSEnumMemberName,
};
use oxc_span::GetSpan;

use starlint_ast::node::{
    self as n, ArrayExpressionNode, ArrayPatternNode, ArrowFunctionExpressionNode,
    AssignmentExpressionNode, AstNode, AwaitExpressionNode, BinaryExpressionNode,
    BindingIdentifierNode, BlockStatementNode, BooleanLiteralNode, BreakStatementNode,
    CallExpressionNode, CatchClauseNode, ChainExpressionNode, ClassNode,
    ComputedMemberExpressionNode, ConditionalExpressionNode, ContinueStatementNode,
    DebuggerStatementNode, DoWhileStatementNode, EmptyStatementNode, ExportAllDeclarationNode,
    ExportDefaultDeclarationNode, ExportNamedDeclarationNode, ExportSpecifierNode,
    ExpressionStatementNode, ForInStatementNode, ForOfStatementNode, ForStatementNode,
    FunctionBodyNode, FunctionNode, IdentifierReferenceNode, IfStatementNode,
    ImportDeclarationNode, ImportSpecifierNode, JSXAttributeNode, JSXElementNode,
    JSXExpressionContainerNode, JSXFragmentNode, JSXOpeningElementNode, JSXSpreadAttributeNode,
    JSXTextNode, LabeledStatementNode, LogicalExpressionNode, MethodDefinitionNode,
    NewExpressionNode, NullLiteralNode, NumericLiteralNode, ObjectExpressionNode,
    ObjectPatternNode, ObjectPropertyNode, PropertyDefinitionNode, RegExpLiteralNode,
    ReturnStatementNode, SequenceExpressionNode, SpreadElementNode, StaticBlockNode,
    StaticMemberExpressionNode, StringLiteralNode, SwitchCaseNode, SwitchStatementNode,
    TSAsExpressionNode, TSEnumDeclarationNode, TSEnumMemberNode, TSInterfaceDeclarationNode,
    TSModuleDeclarationNode, TSNonNullExpressionNode, TSTypeAliasDeclarationNode,
    TSTypeAssertionNode, TSTypeParameterNode, TaggedTemplateExpressionNode, TemplateLiteralNode,
    ThisExpressionNode, ThrowStatementNode, TryStatementNode, UnaryExpressionNode, UnknownNode,
    UpdateExpressionNode, VariableDeclarationNode, VariableDeclaratorNode, WhileStatementNode,
    WithStatementNode,
};
use starlint_ast::operator;
use starlint_ast::tree::AstTree;
use starlint_ast::types::{NodeId, Span};

/// Convert an oxc `Program` into an [`AstTree`].
pub fn convert(program: &oxc::Program<'_>) -> AstTree {
    let mut converter = AstConverter::new();
    converter.convert_program(program);
    converter.tree
}

/// Internal converter state.
struct AstConverter {
    /// The tree being built.
    tree: AstTree,
    /// Stack of parent [`NodeId`]s for tracking ancestry.
    parent_stack: Vec<NodeId>,
}

impl AstConverter {
    /// Create a new converter.
    fn new() -> Self {
        Self {
            tree: AstTree::with_capacity(256),
            parent_stack: Vec::with_capacity(32),
        }
    }

    /// Current parent [`NodeId`] (top of stack), or `None` for the root.
    fn current_parent(&self) -> Option<NodeId> {
        self.parent_stack.last().copied()
    }

    /// Reserve a slot in the tree under the current parent.
    fn reserve(&mut self) -> NodeId {
        self.tree.reserve(self.current_parent())
    }

    /// Convert an oxc span to our span type.
    const fn span(s: oxc_span::Span) -> Span {
        Span::new(s.start, s.end)
    }

    /// Push an [`AstNode::Unknown`] for unhandled node types.
    fn push_unknown(&mut self, s: oxc_span::Span) -> NodeId {
        self.tree.push(
            AstNode::Unknown(UnknownNode {
                span: Self::span(s),
            }),
            self.current_parent(),
        )
    }

    // -----------------------------------------------------------------------
    // Program
    // -----------------------------------------------------------------------

    /// Convert the root program.
    fn convert_program(&mut self, prog: &oxc::Program<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);

        let body: Vec<NodeId> = prog
            .body
            .iter()
            .map(|stmt| self.convert_statement(stmt))
            .collect();

        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::Program(n::ProgramNode {
                span: Self::span(prog.span),
                is_module: prog.source_type.is_module(),
                body: body.into_boxed_slice(),
            }),
        );
        id
    }

    // -----------------------------------------------------------------------
    // Statements
    // -----------------------------------------------------------------------

    /// Dispatch an oxc `Statement` to the appropriate converter.
    #[allow(clippy::too_many_lines)]
    fn convert_statement(&mut self, stmt: &Statement<'_>) -> NodeId {
        match stmt {
            Statement::BlockStatement(it) => self.convert_block_statement(it),
            Statement::BreakStatement(it) => self.tree.push(
                AstNode::BreakStatement(BreakStatementNode {
                    span: Self::span(it.span),
                    label: it.label.as_ref().map(|l| l.name.to_string()),
                }),
                self.current_parent(),
            ),
            Statement::ContinueStatement(it) => self.convert_continue_statement(it),
            Statement::DebuggerStatement(it) => self.convert_debugger_statement(it),
            Statement::DoWhileStatement(it) => self.convert_do_while_statement(it),
            Statement::EmptyStatement(it) => self.convert_empty_statement(it),
            Statement::ExpressionStatement(it) => self.convert_expression_statement(it),
            Statement::ForInStatement(it) => self.convert_for_in_statement(it),
            Statement::ForOfStatement(it) => self.convert_for_of_statement(it),
            Statement::ForStatement(it) => self.convert_for_statement(it),
            Statement::IfStatement(it) => self.convert_if_statement(it),
            Statement::LabeledStatement(it) => self.convert_labeled_statement(it),
            Statement::ReturnStatement(it) => self.convert_return_statement(it),
            Statement::SwitchStatement(it) => self.convert_switch_statement(it),
            Statement::ThrowStatement(it) => self.convert_throw_statement(it),
            Statement::TryStatement(it) => self.convert_try_statement(it),
            Statement::WhileStatement(it) => self.convert_while_statement(it),
            Statement::WithStatement(it) => self.convert_with_statement(it),
            // Declaration variants (inherited via @inherit Declaration)
            Statement::VariableDeclaration(it) => self.convert_variable_declaration(it),
            Statement::FunctionDeclaration(it) => self.convert_function(it),
            Statement::ClassDeclaration(it) => self.convert_class(it),
            Statement::TSTypeAliasDeclaration(it) => self.convert_ts_type_alias_declaration(it),
            Statement::TSInterfaceDeclaration(it) => self.convert_ts_interface_declaration(it),
            Statement::TSEnumDeclaration(it) => self.convert_ts_enum_declaration(it),
            Statement::TSModuleDeclaration(it) => self.convert_ts_module_declaration(it),
            // ModuleDeclaration variants (inherited via @inherit ModuleDeclaration)
            Statement::ImportDeclaration(it) => self.convert_import_declaration(it),
            Statement::ExportAllDeclaration(it) => self.convert_export_all_declaration(it),
            Statement::ExportDefaultDeclaration(it) => self.convert_export_default_declaration(it),
            Statement::ExportNamedDeclaration(it) => self.convert_export_named_declaration(it),
            // Unhandled TS/oxc-specific statement types
            _ => self.push_unknown(stmt.span()),
        }
    }

    /// `{ ... }`
    fn convert_block_statement(&mut self, it: &oxc::BlockStatement<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let body: Vec<NodeId> = it.body.iter().map(|s| self.convert_statement(s)).collect();
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::BlockStatement(BlockStatementNode {
                span: Self::span(it.span),
                body: body.into_boxed_slice(),
            }),
        );
        id
    }

    /// `if (test) { ... } else { ... }`
    fn convert_if_statement(&mut self, it: &oxc::IfStatement<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let test = self.convert_expression(&it.test);
        let consequent = self.convert_statement(&it.consequent);
        let alternate = it.alternate.as_ref().map(|a| self.convert_statement(a));
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::IfStatement(IfStatementNode {
                span: Self::span(it.span),
                test,
                consequent,
                alternate,
            }),
        );
        id
    }

    /// `switch (discriminant) { ... }`
    fn convert_switch_statement(&mut self, it: &oxc::SwitchStatement<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let discriminant = self.convert_expression(&it.discriminant);
        let cases: Vec<NodeId> = it
            .cases
            .iter()
            .map(|c| self.convert_switch_case(c))
            .collect();
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::SwitchStatement(SwitchStatementNode {
                span: Self::span(it.span),
                discriminant,
                cases: cases.into_boxed_slice(),
            }),
        );
        id
    }

    /// Single switch case.
    fn convert_switch_case(&mut self, it: &oxc::SwitchCase<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let test = it.test.as_ref().map(|t| self.convert_expression(t));
        let consequent: Vec<NodeId> = it
            .consequent
            .iter()
            .map(|s| self.convert_statement(s))
            .collect();
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::SwitchCase(SwitchCaseNode {
                span: Self::span(it.span),
                test,
                consequent: consequent.into_boxed_slice(),
            }),
        );
        id
    }

    /// `for (init; test; update) { ... }`
    fn convert_for_statement(&mut self, it: &oxc::ForStatement<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let init = it.init.as_ref().map(|i| self.convert_for_statement_init(i));
        let test = it.test.as_ref().map(|t| self.convert_expression(t));
        let update = it.update.as_ref().map(|u| self.convert_expression(u));
        let body = self.convert_statement(&it.body);
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::ForStatement(ForStatementNode {
                span: Self::span(it.span),
                init,
                test,
                update,
                body,
            }),
        );
        id
    }

    /// For-statement init clause (expression or variable declaration).
    fn convert_for_statement_init(&mut self, it: &ForStatementInit<'_>) -> NodeId {
        match it {
            ForStatementInit::VariableDeclaration(v) => self.convert_variable_declaration(v),
            _ => self.convert_expression(it.to_expression()),
        }
    }

    /// `for (left in right) { ... }`
    fn convert_for_in_statement(&mut self, it: &oxc::ForInStatement<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let left = self.convert_for_statement_left(&it.left);
        let right = self.convert_expression(&it.right);
        let body = self.convert_statement(&it.body);
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::ForInStatement(ForInStatementNode {
                span: Self::span(it.span),
                left,
                right,
                body,
            }),
        );
        id
    }

    /// `for (left of right) { ... }`
    fn convert_for_of_statement(&mut self, it: &oxc::ForOfStatement<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let left = self.convert_for_statement_left(&it.left);
        let right = self.convert_expression(&it.right);
        let body = self.convert_statement(&it.body);
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::ForOfStatement(ForOfStatementNode {
                span: Self::span(it.span),
                is_await: it.r#await,
                left,
                right,
                body,
            }),
        );
        id
    }

    /// For-in/for-of left-hand side.
    fn convert_for_statement_left(&mut self, it: &ForStatementLeft<'_>) -> NodeId {
        match it {
            ForStatementLeft::VariableDeclaration(v) => self.convert_variable_declaration(v),
            ForStatementLeft::AssignmentTargetIdentifier(ident) => self.tree.push(
                AstNode::IdentifierReference(IdentifierReferenceNode {
                    span: Self::span(ident.span),
                    name: ident.name.to_string(),
                }),
                self.current_parent(),
            ),
            _ => self.push_unknown(it.span()),
        }
    }

    /// `while (test) { ... }`
    fn convert_while_statement(&mut self, it: &oxc::WhileStatement<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let test = self.convert_expression(&it.test);
        let body = self.convert_statement(&it.body);
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::WhileStatement(WhileStatementNode {
                span: Self::span(it.span),
                test,
                body,
            }),
        );
        id
    }

    /// `do { ... } while (test)`
    fn convert_do_while_statement(&mut self, it: &oxc::DoWhileStatement<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let body = self.convert_statement(&it.body);
        let test = self.convert_expression(&it.test);
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::DoWhileStatement(DoWhileStatementNode {
                span: Self::span(it.span),
                body,
                test,
            }),
        );
        id
    }

    /// `try { ... } catch { ... } finally { ... }`
    fn convert_try_statement(&mut self, it: &oxc::TryStatement<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let block = self.convert_block_statement(&it.block);
        let handler = it.handler.as_ref().map(|h| self.convert_catch_clause(h));
        let finalizer = it
            .finalizer
            .as_ref()
            .map(|f| self.convert_block_statement(f));
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::TryStatement(TryStatementNode {
                span: Self::span(it.span),
                block,
                handler,
                finalizer,
            }),
        );
        id
    }

    /// `catch (param) { ... }`
    fn convert_catch_clause(&mut self, it: &oxc::CatchClause<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let param = it
            .param
            .as_ref()
            .map(|p| self.convert_binding_pattern(&p.pattern));
        let body = self.convert_block_statement(&it.body);
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::CatchClause(CatchClauseNode {
                span: Self::span(it.span),
                param,
                body,
            }),
        );
        id
    }

    /// `throw argument`
    fn convert_throw_statement(&mut self, it: &oxc::ThrowStatement<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let argument = self.convert_expression(&it.argument);
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::ThrowStatement(ThrowStatementNode {
                span: Self::span(it.span),
                argument,
            }),
        );
        id
    }

    /// `return argument`
    fn convert_return_statement(&mut self, it: &oxc::ReturnStatement<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let argument = it.argument.as_ref().map(|a| self.convert_expression(a));
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::ReturnStatement(ReturnStatementNode {
                span: Self::span(it.span),
                argument,
            }),
        );
        id
    }

    /// `label: body`
    fn convert_labeled_statement(&mut self, it: &oxc::LabeledStatement<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let body = self.convert_statement(&it.body);
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::LabeledStatement(LabeledStatementNode {
                span: Self::span(it.span),
                label: it.label.name.to_string(),
                body,
            }),
        );
        id
    }

    /// `continue label?`
    fn convert_continue_statement(&mut self, it: &oxc::ContinueStatement<'_>) -> NodeId {
        self.tree.push(
            AstNode::ContinueStatement(ContinueStatementNode {
                span: Self::span(it.span),
                label: it.label.as_ref().map(|l| l.name.to_string()),
            }),
            self.current_parent(),
        )
    }

    /// `;`
    fn convert_empty_statement(&mut self, it: &oxc::EmptyStatement) -> NodeId {
        self.tree.push(
            AstNode::EmptyStatement(EmptyStatementNode {
                span: Self::span(it.span),
            }),
            self.current_parent(),
        )
    }

    /// `with (object) { ... }`
    fn convert_with_statement(&mut self, it: &oxc::WithStatement<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let object = self.convert_expression(&it.object);
        let body = self.convert_statement(&it.body);
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::WithStatement(WithStatementNode {
                span: Self::span(it.span),
                object,
                body,
            }),
        );
        id
    }

    /// `debugger;`
    fn convert_debugger_statement(&mut self, it: &oxc::DebuggerStatement) -> NodeId {
        self.tree.push(
            AstNode::DebuggerStatement(DebuggerStatementNode {
                span: Self::span(it.span),
            }),
            self.current_parent(),
        )
    }

    /// Expression statement.
    fn convert_expression_statement(&mut self, it: &oxc::ExpressionStatement<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let expression = self.convert_expression(&it.expression);
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::ExpressionStatement(ExpressionStatementNode {
                span: Self::span(it.span),
                expression,
            }),
        );
        id
    }

    // -----------------------------------------------------------------------
    // Declarations
    // -----------------------------------------------------------------------

    /// Variable declaration.
    fn convert_variable_declaration(&mut self, it: &oxc::VariableDeclaration<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let declarations: Vec<NodeId> = it
            .declarations
            .iter()
            .map(|d| self.convert_variable_declarator(d))
            .collect();
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::VariableDeclaration(VariableDeclarationNode {
                span: Self::span(it.span),
                kind: convert_variable_declaration_kind(it.kind),
                declarations: declarations.into_boxed_slice(),
            }),
        );
        id
    }

    /// Single variable declarator.
    fn convert_variable_declarator(&mut self, it: &oxc::VariableDeclarator<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let binding_id = self.convert_binding_pattern(&it.id);
        let init = it.init.as_ref().map(|i| self.convert_expression(i));
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::VariableDeclarator(VariableDeclaratorNode {
                span: Self::span(it.span),
                id: binding_id,
                init,
            }),
        );
        id
    }

    /// Function declaration or expression.
    fn convert_function(&mut self, it: &oxc::Function<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let func_id = it
            .id
            .as_ref()
            .map(|name| self.convert_binding_identifier(name));
        let params: Vec<NodeId> = it
            .params
            .items
            .iter()
            .map(|p| self.convert_binding_pattern(&p.pattern))
            .collect();
        let body = it.body.as_ref().map(|b| self.convert_function_body(b));
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::Function(FunctionNode {
                span: Self::span(it.span),
                id: func_id,
                params: params.into_boxed_slice(),
                body,
                is_async: it.r#async,
                is_generator: it.generator,
                is_declare: it.declare,
            }),
        );
        id
    }

    /// Function body.
    fn convert_function_body(&mut self, it: &oxc::FunctionBody<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let statements: Vec<NodeId> = it
            .statements
            .iter()
            .map(|s| self.convert_statement(s))
            .collect();
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::FunctionBody(FunctionBodyNode {
                span: Self::span(it.span),
                statements: statements.into_boxed_slice(),
            }),
        );
        id
    }

    /// Class declaration or expression.
    fn convert_class(&mut self, it: &oxc::Class<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let class_id = it
            .id
            .as_ref()
            .map(|name| self.convert_binding_identifier(name));
        let super_class = it
            .super_class
            .as_ref()
            .map(|sc| self.convert_expression(sc));
        let body: Vec<NodeId> = it
            .body
            .body
            .iter()
            .map(|elem| self.convert_class_element(elem))
            .collect();
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::Class(ClassNode {
                span: Self::span(it.span),
                id: class_id,
                super_class,
                body: body.into_boxed_slice(),
                is_declare: it.declare,
                is_abstract: it.r#abstract,
            }),
        );
        id
    }

    /// Class element (method, property, static block, etc.).
    fn convert_class_element(&mut self, elem: &ClassElement<'_>) -> NodeId {
        match elem {
            ClassElement::MethodDefinition(m) => {
                let id = self.reserve();
                self.parent_stack.push(id);
                let key = self.convert_property_key(&m.key);
                let value = self.convert_function(&m.value);
                self.parent_stack.pop();
                self.tree.set(
                    id,
                    AstNode::MethodDefinition(MethodDefinitionNode {
                        span: Self::span(m.span),
                        kind: convert_method_definition_kind(m.kind),
                        key,
                        value,
                        is_static: m.r#static,
                        computed: m.computed,
                        is_accessor: matches!(
                            m.kind,
                            oxc::MethodDefinitionKind::Get | oxc::MethodDefinitionKind::Set
                        ),
                    }),
                );
                id
            }
            ClassElement::PropertyDefinition(p) => {
                let id = self.reserve();
                self.parent_stack.push(id);
                let key = self.convert_property_key(&p.key);
                let value = p.value.as_ref().map(|v| self.convert_expression(v));
                self.parent_stack.pop();
                self.tree.set(
                    id,
                    AstNode::PropertyDefinition(PropertyDefinitionNode {
                        span: Self::span(p.span),
                        key,
                        value,
                        is_static: p.r#static,
                        computed: p.computed,
                        is_declare: p.declare,
                    }),
                );
                id
            }
            ClassElement::StaticBlock(sb) => {
                let id = self.reserve();
                self.parent_stack.push(id);
                let body: Vec<NodeId> = sb.body.iter().map(|s| self.convert_statement(s)).collect();
                self.parent_stack.pop();
                self.tree.set(
                    id,
                    AstNode::StaticBlock(StaticBlockNode {
                        span: Self::span(sb.span),
                        body: body.into_boxed_slice(),
                    }),
                );
                id
            }
            _ => self.push_unknown(elem.span()),
        }
    }

    // -----------------------------------------------------------------------
    // Expressions
    // -----------------------------------------------------------------------

    /// Dispatch an oxc `Expression` to the appropriate converter.
    #[allow(clippy::too_many_lines)]
    fn convert_expression(&mut self, expr: &Expression<'_>) -> NodeId {
        match expr {
            Expression::BooleanLiteral(it) => self.tree.push(
                AstNode::BooleanLiteral(BooleanLiteralNode {
                    span: Self::span(it.span),
                    value: it.value,
                }),
                self.current_parent(),
            ),
            Expression::NullLiteral(it) => self.tree.push(
                AstNode::NullLiteral(NullLiteralNode {
                    span: Self::span(it.span),
                }),
                self.current_parent(),
            ),
            Expression::NumericLiteral(it) => self.tree.push(
                AstNode::NumericLiteral(NumericLiteralNode {
                    span: Self::span(it.span),
                    value: it.value,
                    raw: it
                        .raw
                        .as_ref()
                        .map_or_else(String::new, ToString::to_string),
                }),
                self.current_parent(),
            ),
            Expression::StringLiteral(it) => self.tree.push(
                AstNode::StringLiteral(StringLiteralNode {
                    span: Self::span(it.span),
                    value: it.value.to_string(),
                }),
                self.current_parent(),
            ),
            Expression::RegExpLiteral(it) => self.tree.push(
                AstNode::RegExpLiteral(RegExpLiteralNode {
                    span: Self::span(it.span),
                    pattern: it.regex.pattern.text.to_string(),
                    flags: it.regex.flags.to_string(),
                }),
                self.current_parent(),
            ),
            Expression::TemplateLiteral(it) => self.convert_template_literal(it),
            Expression::Identifier(it) => self.tree.push(
                AstNode::IdentifierReference(IdentifierReferenceNode {
                    span: Self::span(it.span),
                    name: it.name.to_string(),
                }),
                self.current_parent(),
            ),
            Expression::ThisExpression(it) => self.tree.push(
                AstNode::ThisExpression(ThisExpressionNode {
                    span: Self::span(it.span),
                }),
                self.current_parent(),
            ),
            Expression::ArrayExpression(it) => self.convert_array_expression(it),
            Expression::ObjectExpression(it) => self.convert_object_expression(it),
            Expression::ArrowFunctionExpression(it) => self.convert_arrow_function_expression(it),
            Expression::AssignmentExpression(it) => self.convert_assignment_expression(it),
            Expression::AwaitExpression(it) => self.convert_await_expression(it),
            Expression::BinaryExpression(it) => self.convert_binary_expression(it),
            Expression::CallExpression(it) => self.convert_call_expression(it),
            Expression::ChainExpression(it) => self.convert_chain_expression(it),
            Expression::ConditionalExpression(it) => self.convert_conditional_expression(it),
            Expression::LogicalExpression(it) => self.convert_logical_expression(it),
            Expression::NewExpression(it) => self.convert_new_expression(it),
            Expression::SequenceExpression(it) => self.convert_sequence_expression(it),
            Expression::TaggedTemplateExpression(it) => self.convert_tagged_template_expression(it),
            Expression::UnaryExpression(it) => self.convert_unary_expression(it),
            Expression::UpdateExpression(it) => self.convert_update_expression(it),
            Expression::ClassExpression(it) => self.convert_class(it),
            Expression::FunctionExpression(it) => self.convert_function(it),
            Expression::JSXElement(it) => self.convert_jsx_element(it),
            Expression::JSXFragment(it) => self.convert_jsx_fragment(it),
            Expression::TSAsExpression(it) => self.convert_ts_as_expression(it),
            Expression::TSTypeAssertion(it) => self.convert_ts_type_assertion(it),
            Expression::TSNonNullExpression(it) => self.convert_ts_non_null_expression(it),
            // MemberExpression variants (inherited)
            Expression::StaticMemberExpression(it) => self.convert_static_member_expression(it),
            Expression::ComputedMemberExpression(it) => self.convert_computed_member_expression(it),
            // ParenthesizedExpression — unwrap to inner expression
            Expression::ParenthesizedExpression(it) => self.convert_expression(&it.expression),
            // Everything else → Unknown
            _ => self.push_unknown(expr.span()),
        }
    }

    /// Call expression.
    fn convert_call_expression(&mut self, it: &oxc::CallExpression<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let callee = self.convert_expression(&it.callee);
        let arguments: Vec<NodeId> = it
            .arguments
            .iter()
            .map(|a| self.convert_argument(a))
            .collect();
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::CallExpression(CallExpressionNode {
                span: Self::span(it.span),
                callee,
                arguments: arguments.into_boxed_slice(),
                optional: it.optional,
            }),
        );
        id
    }

    /// New expression.
    fn convert_new_expression(&mut self, it: &oxc::NewExpression<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let callee = self.convert_expression(&it.callee);
        let arguments: Vec<NodeId> = it
            .arguments
            .iter()
            .map(|a| self.convert_argument(a))
            .collect();
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::NewExpression(NewExpressionNode {
                span: Self::span(it.span),
                callee,
                arguments: arguments.into_boxed_slice(),
            }),
        );
        id
    }

    /// Argument (expression or spread).
    fn convert_argument(&mut self, it: &Argument<'_>) -> NodeId {
        match it {
            Argument::SpreadElement(s) => {
                let id = self.reserve();
                self.parent_stack.push(id);
                let argument = self.convert_expression(&s.argument);
                self.parent_stack.pop();
                self.tree.set(
                    id,
                    AstNode::SpreadElement(SpreadElementNode {
                        span: Self::span(s.span),
                        argument,
                    }),
                );
                id
            }
            _ => self.convert_expression(it.to_expression()),
        }
    }

    /// Binary expression.
    fn convert_binary_expression(&mut self, it: &oxc::BinaryExpression<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let left = self.convert_expression(&it.left);
        let right = self.convert_expression(&it.right);
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::BinaryExpression(BinaryExpressionNode {
                span: Self::span(it.span),
                operator: convert_binary_operator(it.operator),
                left,
                right,
            }),
        );
        id
    }

    /// Logical expression.
    fn convert_logical_expression(&mut self, it: &oxc::LogicalExpression<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let left = self.convert_expression(&it.left);
        let right = self.convert_expression(&it.right);
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::LogicalExpression(LogicalExpressionNode {
                span: Self::span(it.span),
                operator: convert_logical_operator(it.operator),
                left,
                right,
            }),
        );
        id
    }

    /// Assignment expression.
    fn convert_assignment_expression(&mut self, it: &oxc::AssignmentExpression<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let left = self.convert_assignment_target(&it.left);
        let right = self.convert_expression(&it.right);
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::AssignmentExpression(AssignmentExpressionNode {
                span: Self::span(it.span),
                operator: convert_assignment_operator(it.operator),
                left,
                right,
            }),
        );
        id
    }

    /// Assignment target (LHS of assignment).
    fn convert_assignment_target(&mut self, target: &AssignmentTarget<'_>) -> NodeId {
        match target {
            AssignmentTarget::AssignmentTargetIdentifier(ident) => self.tree.push(
                AstNode::IdentifierReference(IdentifierReferenceNode {
                    span: Self::span(ident.span),
                    name: ident.name.to_string(),
                }),
                self.current_parent(),
            ),
            AssignmentTarget::StaticMemberExpression(m) => self.convert_static_member_expression(m),
            AssignmentTarget::ComputedMemberExpression(m) => {
                self.convert_computed_member_expression(m)
            }
            _ => self.push_unknown(target.span()),
        }
    }

    /// Unary expression.
    fn convert_unary_expression(&mut self, it: &oxc::UnaryExpression<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let argument = self.convert_expression(&it.argument);
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::UnaryExpression(UnaryExpressionNode {
                span: Self::span(it.span),
                operator: convert_unary_operator(it.operator),
                argument,
            }),
        );
        id
    }

    /// Update expression.
    fn convert_update_expression(&mut self, it: &oxc::UpdateExpression<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let argument = match &it.argument {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => self.tree.push(
                AstNode::IdentifierReference(IdentifierReferenceNode {
                    span: Self::span(ident.span),
                    name: ident.name.to_string(),
                }),
                self.current_parent(),
            ),
            SimpleAssignmentTarget::StaticMemberExpression(m) => {
                self.convert_static_member_expression(m)
            }
            SimpleAssignmentTarget::ComputedMemberExpression(m) => {
                self.convert_computed_member_expression(m)
            }
            _ => self.push_unknown(it.argument.span()),
        };
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::UpdateExpression(UpdateExpressionNode {
                span: Self::span(it.span),
                operator: convert_update_operator(it.operator),
                prefix: it.prefix,
                argument,
            }),
        );
        id
    }

    /// Conditional expression.
    fn convert_conditional_expression(&mut self, it: &oxc::ConditionalExpression<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let test = self.convert_expression(&it.test);
        let consequent = self.convert_expression(&it.consequent);
        let alternate = self.convert_expression(&it.alternate);
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::ConditionalExpression(ConditionalExpressionNode {
                span: Self::span(it.span),
                test,
                consequent,
                alternate,
            }),
        );
        id
    }

    /// Sequence expression.
    fn convert_sequence_expression(&mut self, it: &oxc::SequenceExpression<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let expressions: Vec<NodeId> = it
            .expressions
            .iter()
            .map(|e| self.convert_expression(e))
            .collect();
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::SequenceExpression(SequenceExpressionNode {
                span: Self::span(it.span),
                expressions: expressions.into_boxed_slice(),
            }),
        );
        id
    }

    /// Template literal.
    fn convert_template_literal(&mut self, it: &oxc::TemplateLiteral<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let quasis: Vec<String> = it.quasis.iter().map(|q| q.value.raw.to_string()).collect();
        let expressions: Vec<NodeId> = it
            .expressions
            .iter()
            .map(|e| self.convert_expression(e))
            .collect();
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::TemplateLiteral(TemplateLiteralNode {
                span: Self::span(it.span),
                quasis: quasis.into_boxed_slice(),
                expressions: expressions.into_boxed_slice(),
            }),
        );
        id
    }

    /// Tagged template expression.
    fn convert_tagged_template_expression(
        &mut self,
        it: &oxc::TaggedTemplateExpression<'_>,
    ) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let tag = self.convert_expression(&it.tag);
        let quasi = self.convert_template_literal(&it.quasi);
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::TaggedTemplateExpression(TaggedTemplateExpressionNode {
                span: Self::span(it.span),
                tag,
                quasi,
            }),
        );
        id
    }

    /// Array expression.
    fn convert_array_expression(&mut self, it: &oxc::ArrayExpression<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let elements: Vec<NodeId> = it
            .elements
            .iter()
            .map(|e| self.convert_array_expression_element(e))
            .collect();
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::ArrayExpression(ArrayExpressionNode {
                span: Self::span(it.span),
                elements: elements.into_boxed_slice(),
            }),
        );
        id
    }

    /// Array expression element.
    fn convert_array_expression_element(&mut self, elem: &ArrayExpressionElement<'_>) -> NodeId {
        match elem {
            ArrayExpressionElement::SpreadElement(s) => {
                let id = self.reserve();
                self.parent_stack.push(id);
                let argument = self.convert_expression(&s.argument);
                self.parent_stack.pop();
                self.tree.set(
                    id,
                    AstNode::SpreadElement(SpreadElementNode {
                        span: Self::span(s.span),
                        argument,
                    }),
                );
                id
            }
            ArrayExpressionElement::Elision(e) => self.push_unknown(e.span),
            _ => self.convert_expression(elem.to_expression()),
        }
    }

    /// Object expression.
    fn convert_object_expression(&mut self, it: &oxc::ObjectExpression<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let properties: Vec<NodeId> = it
            .properties
            .iter()
            .map(|p| self.convert_object_property_kind(p))
            .collect();
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::ObjectExpression(ObjectExpressionNode {
                span: Self::span(it.span),
                properties: properties.into_boxed_slice(),
            }),
        );
        id
    }

    /// Object property kind (property or spread).
    fn convert_object_property_kind(&mut self, prop: &ObjectPropertyKind<'_>) -> NodeId {
        match prop {
            ObjectPropertyKind::ObjectProperty(p) => {
                let id = self.reserve();
                self.parent_stack.push(id);
                let key = self.convert_property_key(&p.key);
                let value = self.convert_expression(&p.value);
                self.parent_stack.pop();
                self.tree.set(
                    id,
                    AstNode::ObjectProperty(ObjectPropertyNode {
                        span: Self::span(p.span),
                        kind: convert_property_kind(p.kind),
                        key,
                        value,
                        computed: p.computed,
                        shorthand: p.shorthand,
                        method: p.method,
                    }),
                );
                id
            }
            ObjectPropertyKind::SpreadProperty(s) => {
                let id = self.reserve();
                self.parent_stack.push(id);
                let argument = self.convert_expression(&s.argument);
                self.parent_stack.pop();
                self.tree.set(
                    id,
                    AstNode::SpreadElement(SpreadElementNode {
                        span: Self::span(s.span),
                        argument,
                    }),
                );
                id
            }
        }
    }

    /// Arrow function expression.
    fn convert_arrow_function_expression(
        &mut self,
        it: &oxc::ArrowFunctionExpression<'_>,
    ) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let params: Vec<NodeId> = it
            .params
            .items
            .iter()
            .map(|p| self.convert_binding_pattern(&p.pattern))
            .collect();
        let body = self.convert_function_body(&it.body);
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::ArrowFunctionExpression(ArrowFunctionExpressionNode {
                span: Self::span(it.span),
                params: params.into_boxed_slice(),
                body,
                is_async: it.r#async,
                expression: it.expression,
            }),
        );
        id
    }

    /// Await expression.
    fn convert_await_expression(&mut self, it: &oxc::AwaitExpression<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let argument = self.convert_expression(&it.argument);
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::AwaitExpression(AwaitExpressionNode {
                span: Self::span(it.span),
                argument,
            }),
        );
        id
    }

    /// Static member expression (`obj.prop`).
    fn convert_static_member_expression(&mut self, it: &oxc::StaticMemberExpression<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let object = self.convert_expression(&it.object);
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::StaticMemberExpression(StaticMemberExpressionNode {
                span: Self::span(it.span),
                object,
                property: it.property.name.to_string(),
                optional: it.optional,
            }),
        );
        id
    }

    /// Computed member expression (`obj[expr]`).
    fn convert_computed_member_expression(
        &mut self,
        it: &oxc::ComputedMemberExpression<'_>,
    ) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let object = self.convert_expression(&it.object);
        let expression = self.convert_expression(&it.expression);
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::ComputedMemberExpression(ComputedMemberExpressionNode {
                span: Self::span(it.span),
                object,
                expression,
                optional: it.optional,
            }),
        );
        id
    }

    /// Chain expression (`a?.b?.c`).
    fn convert_chain_expression(&mut self, it: &oxc::ChainExpression<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let expression = match &it.expression {
            ChainElement::CallExpression(c) => self.convert_call_expression(c),
            ChainElement::StaticMemberExpression(m) => self.convert_static_member_expression(m),
            ChainElement::ComputedMemberExpression(m) => self.convert_computed_member_expression(m),
            _ => self.push_unknown(it.expression.span()),
        };
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::ChainExpression(ChainExpressionNode {
                span: Self::span(it.span),
                expression,
            }),
        );
        id
    }

    // -----------------------------------------------------------------------
    // Patterns / Bindings
    // -----------------------------------------------------------------------

    /// Binding pattern → node.
    fn convert_binding_pattern(&mut self, pat: &BindingPattern<'_>) -> NodeId {
        match pat {
            BindingPattern::BindingIdentifier(ident) => self.convert_binding_identifier(ident),
            BindingPattern::ObjectPattern(obj) => {
                let id = self.reserve();
                self.parent_stack.push(id);
                let properties: Vec<NodeId> = obj
                    .properties
                    .iter()
                    .map(|p| {
                        let prop_id = self.reserve();
                        self.parent_stack.push(prop_id);
                        let key = self.convert_property_key(&p.key);
                        let value = self.convert_binding_pattern(&p.value);
                        self.parent_stack.pop();
                        self.tree.set(
                            prop_id,
                            AstNode::ObjectProperty(ObjectPropertyNode {
                                span: Self::span(p.span),
                                kind: operator::PropertyKind::Init,
                                key,
                                value,
                                computed: p.computed,
                                shorthand: p.shorthand,
                                method: false,
                            }),
                        );
                        prop_id
                    })
                    .collect();
                let rest = obj
                    .rest
                    .as_ref()
                    .map(|r| self.convert_binding_pattern(&r.argument));
                self.parent_stack.pop();
                self.tree.set(
                    id,
                    AstNode::ObjectPattern(ObjectPatternNode {
                        span: Self::span(obj.span),
                        properties: properties.into_boxed_slice(),
                        rest,
                    }),
                );
                id
            }
            BindingPattern::ArrayPattern(arr) => {
                let id = self.reserve();
                self.parent_stack.push(id);
                let elements: Vec<Option<NodeId>> = arr
                    .elements
                    .iter()
                    .map(|e| e.as_ref().map(|p| self.convert_binding_pattern(p)))
                    .collect();
                let rest = arr
                    .rest
                    .as_ref()
                    .map(|r| self.convert_binding_pattern(&r.argument));
                self.parent_stack.pop();
                self.tree.set(
                    id,
                    AstNode::ArrayPattern(ArrayPatternNode {
                        span: Self::span(arr.span),
                        elements: elements.into_boxed_slice(),
                        rest,
                    }),
                );
                id
            }
            BindingPattern::AssignmentPattern(ap) => {
                // Assignment pattern in destructuring (e.g. `{ a = 1 }`) —
                // convert the left side, ignoring the default value for now.
                self.convert_binding_pattern(&ap.left)
            }
        }
    }

    /// Binding identifier.
    fn convert_binding_identifier(&mut self, it: &oxc::BindingIdentifier<'_>) -> NodeId {
        self.tree.push(
            AstNode::BindingIdentifier(BindingIdentifierNode {
                span: Self::span(it.span),
                name: it.name.to_string(),
            }),
            self.current_parent(),
        )
    }

    // -----------------------------------------------------------------------
    // Modules
    // -----------------------------------------------------------------------

    /// Import declaration.
    fn convert_import_declaration(&mut self, it: &oxc::ImportDeclaration<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let specifiers: Vec<NodeId> = it
            .specifiers
            .as_ref()
            .map(|specs| {
                specs
                    .iter()
                    .map(|s| self.convert_import_specifier(s))
                    .collect()
            })
            .unwrap_or_default();
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::ImportDeclaration(ImportDeclarationNode {
                span: Self::span(it.span),
                source: it.source.value.to_string(),
                specifiers: specifiers.into_boxed_slice(),
                import_kind_is_type: it.import_kind.is_type(),
            }),
        );
        id
    }

    /// Import specifier.
    fn convert_import_specifier(&mut self, spec: &ImportDeclarationSpecifier<'_>) -> NodeId {
        match spec {
            ImportDeclarationSpecifier::ImportSpecifier(s) => self.tree.push(
                AstNode::ImportSpecifier(ImportSpecifierNode {
                    span: Self::span(s.span),
                    imported: s.imported.name().to_string(),
                    local: s.local.name.to_string(),
                    is_type: s.import_kind.is_type(),
                }),
                self.current_parent(),
            ),
            ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => self.tree.push(
                AstNode::ImportSpecifier(ImportSpecifierNode {
                    span: Self::span(s.span),
                    imported: "default".to_owned(),
                    local: s.local.name.to_string(),
                    is_type: false,
                }),
                self.current_parent(),
            ),
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => self.tree.push(
                AstNode::ImportSpecifier(ImportSpecifierNode {
                    span: Self::span(s.span),
                    imported: "*".to_owned(),
                    local: s.local.name.to_string(),
                    is_type: false,
                }),
                self.current_parent(),
            ),
        }
    }

    /// Export named declaration.
    fn convert_export_named_declaration(&mut self, it: &oxc::ExportNamedDeclaration<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let declaration = it.declaration.as_ref().map(|d| self.convert_declaration(d));
        let specifiers: Vec<NodeId> = it
            .specifiers
            .iter()
            .map(|s| {
                self.tree.push(
                    AstNode::ExportSpecifier(ExportSpecifierNode {
                        span: Self::span(s.span),
                        local: s.local.name().to_string(),
                        exported: s.exported.name().to_string(),
                    }),
                    self.current_parent(),
                )
            })
            .collect();
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::ExportNamedDeclaration(ExportNamedDeclarationNode {
                span: Self::span(it.span),
                declaration,
                specifiers: specifiers.into_boxed_slice(),
                source: it.source.as_ref().map(|s| s.value.to_string()),
            }),
        );
        id
    }

    /// Export default declaration.
    fn convert_export_default_declaration(
        &mut self,
        it: &oxc::ExportDefaultDeclaration<'_>,
    ) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let declaration = match &it.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(f) => self.convert_function(f),
            ExportDefaultDeclarationKind::ClassDeclaration(c) => self.convert_class(c),
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(t) => {
                self.convert_ts_interface_declaration(t)
            }
            _ => {
                if let Some(expr) = it.declaration.as_expression() {
                    self.convert_expression(expr)
                } else {
                    self.push_unknown(it.declaration.span())
                }
            }
        };
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::ExportDefaultDeclaration(ExportDefaultDeclarationNode {
                span: Self::span(it.span),
                declaration,
            }),
        );
        id
    }

    /// Export all declaration.
    fn convert_export_all_declaration(&mut self, it: &oxc::ExportAllDeclaration<'_>) -> NodeId {
        self.tree.push(
            AstNode::ExportAllDeclaration(ExportAllDeclarationNode {
                span: Self::span(it.span),
                source: it.source.value.to_string(),
                exported: it.exported.as_ref().map(|e| e.name().to_string()),
            }),
            self.current_parent(),
        )
    }

    /// Declaration (subset of Statement — used by export named).
    fn convert_declaration(&mut self, decl: &oxc::Declaration<'_>) -> NodeId {
        match decl {
            oxc::Declaration::VariableDeclaration(v) => self.convert_variable_declaration(v),
            oxc::Declaration::FunctionDeclaration(f) => self.convert_function(f),
            oxc::Declaration::ClassDeclaration(c) => self.convert_class(c),
            oxc::Declaration::TSTypeAliasDeclaration(t) => {
                self.convert_ts_type_alias_declaration(t)
            }
            oxc::Declaration::TSInterfaceDeclaration(t) => self.convert_ts_interface_declaration(t),
            oxc::Declaration::TSEnumDeclaration(t) => self.convert_ts_enum_declaration(t),
            oxc::Declaration::TSModuleDeclaration(t) => self.convert_ts_module_declaration(t),
            _ => self.push_unknown(decl.span()),
        }
    }

    // -----------------------------------------------------------------------
    // JSX
    // -----------------------------------------------------------------------

    /// JSX element.
    fn convert_jsx_element(&mut self, it: &oxc::JSXElement<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let self_closing = it.closing_element.is_none();
        let opening_element = self.convert_jsx_opening_element(&it.opening_element, self_closing);
        let children: Vec<NodeId> = it
            .children
            .iter()
            .map(|c| self.convert_jsx_child(c))
            .collect();
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::JSXElement(JSXElementNode {
                span: Self::span(it.span),
                opening_element,
                children: children.into_boxed_slice(),
            }),
        );
        id
    }

    /// JSX opening element.
    fn convert_jsx_opening_element(
        &mut self,
        it: &oxc::JSXOpeningElement<'_>,
        self_closing: bool,
    ) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let attributes: Vec<NodeId> = it
            .attributes
            .iter()
            .map(|a| self.convert_jsx_attribute_item(a))
            .collect();
        self.parent_stack.pop();
        let name = jsx_element_name_to_string(&it.name);
        self.tree.set(
            id,
            AstNode::JSXOpeningElement(JSXOpeningElementNode {
                span: Self::span(it.span),
                name,
                attributes: attributes.into_boxed_slice(),
                self_closing,
            }),
        );
        id
    }

    /// JSX fragment.
    fn convert_jsx_fragment(&mut self, it: &oxc::JSXFragment<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let children: Vec<NodeId> = it
            .children
            .iter()
            .map(|c| self.convert_jsx_child(c))
            .collect();
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::JSXFragment(JSXFragmentNode {
                span: Self::span(it.span),
                children: children.into_boxed_slice(),
            }),
        );
        id
    }

    /// JSX attribute item.
    fn convert_jsx_attribute_item(&mut self, item: &oxc::JSXAttributeItem<'_>) -> NodeId {
        match item {
            oxc::JSXAttributeItem::Attribute(attr) => {
                let id = self.reserve();
                self.parent_stack.push(id);
                let value = attr
                    .value
                    .as_ref()
                    .map(|v| self.convert_jsx_attribute_value(v));
                self.parent_stack.pop();
                let name = jsx_attribute_name_to_string(&attr.name);
                self.tree.set(
                    id,
                    AstNode::JSXAttribute(JSXAttributeNode {
                        span: Self::span(attr.span),
                        name,
                        value,
                    }),
                );
                id
            }
            oxc::JSXAttributeItem::SpreadAttribute(spread) => {
                let id = self.reserve();
                self.parent_stack.push(id);
                let argument = self.convert_expression(&spread.argument);
                self.parent_stack.pop();
                self.tree.set(
                    id,
                    AstNode::JSXSpreadAttribute(JSXSpreadAttributeNode {
                        span: Self::span(spread.span),
                        argument,
                    }),
                );
                id
            }
        }
    }

    /// JSX attribute value.
    fn convert_jsx_attribute_value(&mut self, value: &oxc::JSXAttributeValue<'_>) -> NodeId {
        match value {
            oxc::JSXAttributeValue::StringLiteral(s) => self.tree.push(
                AstNode::StringLiteral(StringLiteralNode {
                    span: Self::span(s.span),
                    value: s.value.to_string(),
                }),
                self.current_parent(),
            ),
            oxc::JSXAttributeValue::ExpressionContainer(ec) => {
                self.convert_jsx_expression_container(ec)
            }
            oxc::JSXAttributeValue::Element(el) => self.convert_jsx_element(el),
            oxc::JSXAttributeValue::Fragment(frag) => self.convert_jsx_fragment(frag),
        }
    }

    /// JSX expression container.
    fn convert_jsx_expression_container(&mut self, it: &oxc::JSXExpressionContainer<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let expression = match &it.expression {
            oxc::JSXExpression::EmptyExpression(_) => None,
            _ => it
                .expression
                .as_expression()
                .map(|expr| self.convert_expression(expr)),
        };
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::JSXExpressionContainer(JSXExpressionContainerNode {
                span: Self::span(it.span),
                expression,
            }),
        );
        id
    }

    /// JSX child.
    fn convert_jsx_child(&mut self, child: &oxc::JSXChild<'_>) -> NodeId {
        match child {
            oxc::JSXChild::Text(t) => self.tree.push(
                AstNode::JSXText(JSXTextNode {
                    span: Self::span(t.span),
                    value: t.value.to_string(),
                }),
                self.current_parent(),
            ),
            oxc::JSXChild::Element(el) => self.convert_jsx_element(el),
            oxc::JSXChild::Fragment(frag) => self.convert_jsx_fragment(frag),
            oxc::JSXChild::ExpressionContainer(ec) => self.convert_jsx_expression_container(ec),
            oxc::JSXChild::Spread(s) => {
                let id = self.reserve();
                self.parent_stack.push(id);
                let argument = self.convert_expression(&s.expression);
                self.parent_stack.pop();
                self.tree.set(
                    id,
                    AstNode::SpreadElement(SpreadElementNode {
                        span: Self::span(s.span),
                        argument,
                    }),
                );
                id
            }
        }
    }

    // -----------------------------------------------------------------------
    // TypeScript
    // -----------------------------------------------------------------------

    /// TS type alias.
    fn convert_ts_type_alias_declaration(
        &mut self,
        it: &oxc::TSTypeAliasDeclaration<'_>,
    ) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let type_id = self.convert_binding_identifier(&it.id);
        let type_parameters: Vec<NodeId> = it
            .type_parameters
            .as_ref()
            .map(|tp| {
                tp.params
                    .iter()
                    .map(|p| self.convert_ts_type_parameter(p))
                    .collect()
            })
            .unwrap_or_default();
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::TSTypeAliasDeclaration(TSTypeAliasDeclarationNode {
                span: Self::span(it.span),
                id: type_id,
                type_parameters: type_parameters.into_boxed_slice(),
            }),
        );
        id
    }

    /// TS interface.
    fn convert_ts_interface_declaration(&mut self, it: &oxc::TSInterfaceDeclaration<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let intf_id = self.convert_binding_identifier(&it.id);
        // Interface body members are not deeply converted yet
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::TSInterfaceDeclaration(TSInterfaceDeclarationNode {
                span: Self::span(it.span),
                id: intf_id,
                body: Box::new([]),
            }),
        );
        id
    }

    /// TS enum.
    fn convert_ts_enum_declaration(&mut self, it: &oxc::TSEnumDeclaration<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let enum_id = self.convert_binding_identifier(&it.id);
        let members: Vec<NodeId> = it
            .body
            .members
            .iter()
            .map(|m| {
                let mid = self.reserve();
                self.parent_stack.push(mid);
                let member_id = self.convert_ts_enum_member_name(&m.id);
                let initializer = m.initializer.as_ref().map(|i| self.convert_expression(i));
                self.parent_stack.pop();
                self.tree.set(
                    mid,
                    AstNode::TSEnumMember(TSEnumMemberNode {
                        span: Self::span(m.span),
                        id: member_id,
                        initializer,
                    }),
                );
                mid
            })
            .collect();
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::TSEnumDeclaration(TSEnumDeclarationNode {
                span: Self::span(it.span),
                id: enum_id,
                members: members.into_boxed_slice(),
                is_const: it.r#const,
                is_declare: it.declare,
            }),
        );
        id
    }

    /// TS module/namespace declaration.
    fn convert_ts_module_declaration(&mut self, it: &oxc::TSModuleDeclaration<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let mod_id = match &it.id {
            oxc::TSModuleDeclarationName::Identifier(ident) => self.tree.push(
                AstNode::BindingIdentifier(BindingIdentifierNode {
                    span: Self::span(ident.span),
                    name: ident.name.to_string(),
                }),
                self.current_parent(),
            ),
            oxc::TSModuleDeclarationName::StringLiteral(s) => self.tree.push(
                AstNode::StringLiteral(StringLiteralNode {
                    span: Self::span(s.span),
                    value: s.value.to_string(),
                }),
                self.current_parent(),
            ),
        };
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::TSModuleDeclaration(TSModuleDeclarationNode {
                span: Self::span(it.span),
                id: mod_id,
                body: None,
                is_declare: it.declare,
            }),
        );
        id
    }

    /// TS `as` expression.
    fn convert_ts_as_expression(&mut self, it: &oxc::TSAsExpression<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let expression = self.convert_expression(&it.expression);
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::TSAsExpression(TSAsExpressionNode {
                span: Self::span(it.span),
                expression,
            }),
        );
        id
    }

    /// TS type assertion (`<Type>expr`).
    fn convert_ts_type_assertion(&mut self, it: &oxc::TSTypeAssertion<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let expression = self.convert_expression(&it.expression);
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::TSTypeAssertion(TSTypeAssertionNode {
                span: Self::span(it.span),
                expression,
            }),
        );
        id
    }

    /// TS non-null assertion (`expr!`).
    fn convert_ts_non_null_expression(&mut self, it: &oxc::TSNonNullExpression<'_>) -> NodeId {
        let id = self.reserve();
        self.parent_stack.push(id);
        let expression = self.convert_expression(&it.expression);
        self.parent_stack.pop();
        self.tree.set(
            id,
            AstNode::TSNonNullExpression(TSNonNullExpressionNode {
                span: Self::span(it.span),
                expression,
            }),
        );
        id
    }

    /// TS type parameter.
    fn convert_ts_type_parameter(&mut self, it: &oxc::TSTypeParameter<'_>) -> NodeId {
        self.tree.push(
            AstNode::TSTypeParameter(TSTypeParameterNode {
                span: Self::span(it.span),
                name: it.name.name.to_string(),
                constraint: None,
                default: None,
            }),
            self.current_parent(),
        )
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Convert a `PropertyKey` to a node.
    ///
    /// `PropertyKey` can be an identifier, string, number, or computed expression.
    fn convert_property_key(&mut self, key: &PropertyKey<'_>) -> NodeId {
        match key {
            PropertyKey::StaticIdentifier(id) => self.tree.push(
                AstNode::IdentifierReference(IdentifierReferenceNode {
                    span: Self::span(id.span),
                    name: id.name.to_string(),
                }),
                self.current_parent(),
            ),
            PropertyKey::PrivateIdentifier(id) => self.tree.push(
                AstNode::IdentifierReference(IdentifierReferenceNode {
                    span: Self::span(id.span),
                    name: id.name.to_string(),
                }),
                self.current_parent(),
            ),
            _ => {
                if let Some(expr) = key.as_expression() {
                    self.convert_expression(expr)
                } else {
                    self.push_unknown(key.span())
                }
            }
        }
    }

    /// Convert a `TSEnumMemberName` to a node.
    fn convert_ts_enum_member_name(&mut self, name: &TSEnumMemberName<'_>) -> NodeId {
        match name {
            TSEnumMemberName::Identifier(id) => self.tree.push(
                AstNode::IdentifierReference(IdentifierReferenceNode {
                    span: Self::span(id.span),
                    name: id.name.to_string(),
                }),
                self.current_parent(),
            ),
            TSEnumMemberName::String(s) | TSEnumMemberName::ComputedString(s) => self.tree.push(
                AstNode::StringLiteral(StringLiteralNode {
                    span: Self::span(s.span),
                    value: s.value.to_string(),
                }),
                self.current_parent(),
            ),
            TSEnumMemberName::ComputedTemplateString(t) => self.convert_template_literal(t),
        }
    }
}

// ---------------------------------------------------------------------------
// Free-standing conversion helpers
// ---------------------------------------------------------------------------

/// Convert an oxc `BinaryOperator` to our type.
const fn convert_binary_operator(op: oxc::BinaryOperator) -> operator::BinaryOperator {
    use oxc::BinaryOperator as Oxc;
    match op {
        Oxc::Equality => operator::BinaryOperator::Equality,
        Oxc::Inequality => operator::BinaryOperator::Inequality,
        Oxc::StrictEquality => operator::BinaryOperator::StrictEquality,
        Oxc::StrictInequality => operator::BinaryOperator::StrictInequality,
        Oxc::LessThan => operator::BinaryOperator::LessThan,
        Oxc::LessEqualThan => operator::BinaryOperator::LessEqualThan,
        Oxc::GreaterThan => operator::BinaryOperator::GreaterThan,
        Oxc::GreaterEqualThan => operator::BinaryOperator::GreaterEqualThan,
        Oxc::Addition => operator::BinaryOperator::Addition,
        Oxc::Subtraction => operator::BinaryOperator::Subtraction,
        Oxc::Multiplication => operator::BinaryOperator::Multiplication,
        Oxc::Division => operator::BinaryOperator::Division,
        Oxc::Remainder => operator::BinaryOperator::Remainder,
        Oxc::Exponential => operator::BinaryOperator::Exponential,
        Oxc::ShiftLeft => operator::BinaryOperator::ShiftLeft,
        Oxc::ShiftRight => operator::BinaryOperator::ShiftRight,
        Oxc::ShiftRightZeroFill => operator::BinaryOperator::ShiftRightZeroFill,
        Oxc::BitwiseOR => operator::BinaryOperator::BitwiseOR,
        Oxc::BitwiseXOR => operator::BinaryOperator::BitwiseXOR,
        Oxc::BitwiseAnd => operator::BinaryOperator::BitwiseAnd,
        Oxc::In => operator::BinaryOperator::In,
        Oxc::Instanceof => operator::BinaryOperator::Instanceof,
    }
}

/// Convert an oxc `LogicalOperator` to our type.
const fn convert_logical_operator(op: oxc::LogicalOperator) -> operator::LogicalOperator {
    use oxc::LogicalOperator as Oxc;
    match op {
        Oxc::Or => operator::LogicalOperator::Or,
        Oxc::And => operator::LogicalOperator::And,
        Oxc::Coalesce => operator::LogicalOperator::Coalesce,
    }
}

/// Convert an oxc `UnaryOperator` to our type.
const fn convert_unary_operator(op: oxc::UnaryOperator) -> operator::UnaryOperator {
    use oxc::UnaryOperator as Oxc;
    match op {
        Oxc::UnaryPlus => operator::UnaryOperator::UnaryPlus,
        Oxc::UnaryNegation => operator::UnaryOperator::UnaryNegation,
        Oxc::LogicalNot => operator::UnaryOperator::LogicalNot,
        Oxc::BitwiseNot => operator::UnaryOperator::BitwiseNot,
        Oxc::Typeof => operator::UnaryOperator::Typeof,
        Oxc::Void => operator::UnaryOperator::Void,
        Oxc::Delete => operator::UnaryOperator::Delete,
    }
}

/// Convert an oxc `UpdateOperator` to our type.
const fn convert_update_operator(op: oxc::UpdateOperator) -> operator::UpdateOperator {
    use oxc::UpdateOperator as Oxc;
    match op {
        Oxc::Increment => operator::UpdateOperator::Increment,
        Oxc::Decrement => operator::UpdateOperator::Decrement,
    }
}

/// Convert an oxc `AssignmentOperator` to our type.
const fn convert_assignment_operator(op: oxc::AssignmentOperator) -> operator::AssignmentOperator {
    use oxc::AssignmentOperator as Oxc;
    match op {
        Oxc::Assign => operator::AssignmentOperator::Assign,
        Oxc::Addition => operator::AssignmentOperator::Addition,
        Oxc::Subtraction => operator::AssignmentOperator::Subtraction,
        Oxc::Multiplication => operator::AssignmentOperator::Multiplication,
        Oxc::Division => operator::AssignmentOperator::Division,
        Oxc::Remainder => operator::AssignmentOperator::Remainder,
        Oxc::Exponential => operator::AssignmentOperator::Exponential,
        Oxc::ShiftLeft => operator::AssignmentOperator::ShiftLeft,
        Oxc::ShiftRight => operator::AssignmentOperator::ShiftRight,
        Oxc::ShiftRightZeroFill => operator::AssignmentOperator::ShiftRightZeroFill,
        Oxc::BitwiseOR => operator::AssignmentOperator::BitwiseOR,
        Oxc::BitwiseXOR => operator::AssignmentOperator::BitwiseXOR,
        Oxc::BitwiseAnd => operator::AssignmentOperator::BitwiseAnd,
        Oxc::LogicalOr => operator::AssignmentOperator::LogicalOr,
        Oxc::LogicalAnd => operator::AssignmentOperator::LogicalAnd,
        Oxc::LogicalNullish => operator::AssignmentOperator::LogicalNullish,
    }
}

/// Convert an oxc `VariableDeclarationKind` to our type.
const fn convert_variable_declaration_kind(
    kind: oxc::VariableDeclarationKind,
) -> operator::VariableDeclarationKind {
    match kind {
        oxc::VariableDeclarationKind::Var => operator::VariableDeclarationKind::Var,
        oxc::VariableDeclarationKind::Let => operator::VariableDeclarationKind::Let,
        oxc::VariableDeclarationKind::Const => operator::VariableDeclarationKind::Const,
        oxc::VariableDeclarationKind::Using => operator::VariableDeclarationKind::Using,
        oxc::VariableDeclarationKind::AwaitUsing => operator::VariableDeclarationKind::AwaitUsing,
    }
}

/// Convert an oxc `PropertyKind` to our type.
const fn convert_property_kind(kind: oxc::PropertyKind) -> operator::PropertyKind {
    match kind {
        oxc::PropertyKind::Init => operator::PropertyKind::Init,
        oxc::PropertyKind::Get => operator::PropertyKind::Get,
        oxc::PropertyKind::Set => operator::PropertyKind::Set,
    }
}

/// Convert an oxc `MethodDefinitionKind` to our type.
const fn convert_method_definition_kind(
    kind: oxc::MethodDefinitionKind,
) -> operator::MethodDefinitionKind {
    match kind {
        oxc::MethodDefinitionKind::Method => operator::MethodDefinitionKind::Method,
        oxc::MethodDefinitionKind::Constructor => operator::MethodDefinitionKind::Constructor,
        oxc::MethodDefinitionKind::Get => operator::MethodDefinitionKind::Get,
        oxc::MethodDefinitionKind::Set => operator::MethodDefinitionKind::Set,
    }
}

/// Convert JSX element name to a string.
fn jsx_element_name_to_string(name: &oxc::JSXElementName<'_>) -> String {
    match name {
        oxc::JSXElementName::Identifier(id) => id.name.to_string(),
        oxc::JSXElementName::IdentifierReference(id) => id.name.to_string(),
        oxc::JSXElementName::NamespacedName(ns) => {
            format!("{}:{}", ns.namespace.name, ns.name.name)
        }
        oxc::JSXElementName::MemberExpression(me) => jsx_member_expression_to_string(me),
        oxc::JSXElementName::ThisExpression(_) => "this".to_owned(),
    }
}

/// Convert JSX member expression chain to dotted string.
fn jsx_member_expression_to_string(me: &oxc::JSXMemberExpression<'_>) -> String {
    let object = match &me.object {
        oxc::JSXMemberExpressionObject::IdentifierReference(id) => id.name.to_string(),
        oxc::JSXMemberExpressionObject::MemberExpression(inner) => {
            jsx_member_expression_to_string(inner)
        }
        oxc::JSXMemberExpressionObject::ThisExpression(_) => "this".to_owned(),
    };
    format!("{object}.{}", me.property.name)
}

/// Convert JSX attribute name to string.
fn jsx_attribute_name_to_string(name: &oxc::JSXAttributeName<'_>) -> String {
    match name {
        oxc::JSXAttributeName::Identifier(id) => id.name.to_string(),
        oxc::JSXAttributeName::NamespacedName(ns) => {
            format!("{}:{}", ns.namespace.name, ns.name.name)
        }
    }
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    use starlint_ast::node::AstNode;
    use starlint_ast::node_type::AstNodeType;

    use super::convert;

    /// Parse source and convert to `AstTree`.
    fn parse_and_convert(source: &str) -> starlint_ast::tree::AstTree {
        let allocator = Allocator::default();
        let source_type = SourceType::mjs();
        let parsed = Parser::new(&allocator, source, source_type).parse();
        convert(&parsed.program)
    }

    #[test]
    fn empty_program() {
        let tree = parse_and_convert("");
        assert_eq!(tree.len(), 1, "just the Program node");
        assert!(
            matches!(
                tree.get(starlint_ast::NodeId::ROOT),
                Some(AstNode::Program(_))
            ),
            "root is Program"
        );
    }

    #[test]
    fn debugger_statement() {
        let tree = parse_and_convert("debugger;");
        // Program → DebuggerStatement
        assert_eq!(tree.len(), 2, "Program + DebuggerStatement");
        assert_eq!(
            tree.node_type(starlint_ast::NodeId(1)),
            Some(AstNodeType::DebuggerStatement),
            "second node is DebuggerStatement"
        );
    }

    #[test]
    fn variable_declaration() {
        let tree = parse_and_convert("const x = 1;");
        // Program → VariableDeclaration → VariableDeclarator → (BindingIdentifier, NumericLiteral)
        assert!(tree.len() >= 4, "should have multiple nodes");
        assert_eq!(
            tree.node_type(starlint_ast::NodeId(1)),
            Some(AstNodeType::VariableDeclaration),
            "should have VariableDeclaration"
        );
    }

    #[test]
    fn if_statement() {
        let tree = parse_and_convert("if (true) { x; } else { y; }");
        assert_eq!(
            tree.node_type(starlint_ast::NodeId(1)),
            Some(AstNodeType::IfStatement),
            "should have IfStatement"
        );
    }

    #[test]
    fn function_declaration() {
        let tree = parse_and_convert("function foo(a, b) { return a; }");
        assert_eq!(
            tree.node_type(starlint_ast::NodeId(1)),
            Some(AstNodeType::Function),
            "should have Function"
        );
    }

    #[test]
    fn call_expression() {
        let tree = parse_and_convert("foo(1, 2);");
        // Program → ExpressionStatement → CallExpression → ...
        assert_eq!(
            tree.node_type(starlint_ast::NodeId(2)),
            Some(AstNodeType::CallExpression),
            "should have CallExpression"
        );
    }

    #[test]
    fn binary_expression() {
        let tree = parse_and_convert("x == y;");
        // Program → ExpressionStatement → BinaryExpression → ...
        assert_eq!(
            tree.node_type(starlint_ast::NodeId(2)),
            Some(AstNodeType::BinaryExpression),
            "should have BinaryExpression"
        );
    }

    #[test]
    fn import_declaration() {
        let tree = parse_and_convert("import { foo } from 'bar';");
        assert_eq!(
            tree.node_type(starlint_ast::NodeId(1)),
            Some(AstNodeType::ImportDeclaration),
            "should have ImportDeclaration"
        );
    }

    #[test]
    fn jsx_element() {
        let allocator = Allocator::default();
        let source_type = SourceType::jsx();
        let source = "<div className='x'>hello</div>;";
        let parsed = Parser::new(&allocator, source, source_type).parse();
        let tree = convert(&parsed.program);
        // Should contain JSXElement and JSXOpeningElement
        let has_jsx = tree
            .iter()
            .any(|(_, node)| matches!(node, AstNode::JSXElement(_)));
        assert!(has_jsx, "should have JSXElement");
    }

    #[test]
    fn parent_pointers_consistent() {
        let tree = parse_and_convert("const x = foo(1);");
        // Every non-root node should have a parent
        for (id, _) in tree.iter() {
            if id == starlint_ast::NodeId::ROOT {
                assert!(tree.parent(id).is_none(), "root has no parent");
            } else {
                assert!(
                    tree.parent(id).is_some(),
                    "non-root node {id:?} should have parent"
                );
            }
        }
    }

    #[test]
    fn children_reference_valid_nodes() {
        let tree = parse_and_convert("function f(x) { if (x) { return x + 1; } }");
        for (id, _) in tree.iter() {
            for child_id in tree.children(id) {
                assert!(
                    tree.get(child_id).is_some(),
                    "child {child_id:?} of {id:?} should exist in tree"
                );
            }
        }
    }

    #[test]
    fn no_unknown_in_common_code() {
        let tree =
            parse_and_convert("const x = 1; let y = 'hello'; function f(a) { return a + x; }");
        let unknown_count = tree
            .iter()
            .filter(|(_, node)| matches!(node, AstNode::Unknown(_)))
            .count();
        assert_eq!(
            unknown_count, 0,
            "common JS code should produce no Unknown nodes"
        );
    }
}
