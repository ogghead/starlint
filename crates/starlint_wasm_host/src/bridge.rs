//! Bridge between oxc AST and WIT plugin types.
//!
//! The [`NodeCollector`] traverses the oxc AST and converts matching nodes
//! to the simplified, stable WIT representation for WASM plugins.

use serde::{Deserialize, Serialize};

use starlint_plugin_sdk::diagnostic::Span;

/// Flags indicating which node types a plugin is interested in.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeInterest {
    /// Import declarations.
    pub import_declaration: bool,
    /// `export default ...` declarations.
    pub export_default_declaration: bool,
    /// `export { ... }` declarations.
    pub export_named_declaration: bool,
    /// Call expressions.
    pub call_expression: bool,
    /// Member expressions.
    pub member_expression: bool,
    /// Identifier references.
    pub identifier_reference: bool,
    /// Arrow function expressions.
    pub arrow_function_expression: bool,
    /// Function declarations.
    pub function_declaration: bool,
    /// Variable declarations.
    pub variable_declaration: bool,
    /// String literals.
    pub string_literal: bool,
    /// Object expressions.
    pub object_expression: bool,
    /// Array expressions.
    pub array_expression: bool,
    /// Debugger statements.
    pub debugger_statement: bool,
}

impl NodeInterest {
    /// Check if any interest flag is set.
    #[must_use]
    pub fn any(&self) -> bool {
        self.import_declaration
            || self.export_default_declaration
            || self.export_named_declaration
            || self.call_expression
            || self.member_expression
            || self.identifier_reference
            || self.arrow_function_expression
            || self.function_declaration
            || self.variable_declaration
            || self.string_literal
            || self.object_expression
            || self.array_expression
            || self.debugger_statement
    }

    /// Compute the union of two interest sets.
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        Self {
            import_declaration: self.import_declaration || other.import_declaration,
            export_default_declaration: self.export_default_declaration
                || other.export_default_declaration,
            export_named_declaration: self.export_named_declaration
                || other.export_named_declaration,
            call_expression: self.call_expression || other.call_expression,
            member_expression: self.member_expression || other.member_expression,
            identifier_reference: self.identifier_reference || other.identifier_reference,
            arrow_function_expression: self.arrow_function_expression
                || other.arrow_function_expression,
            function_declaration: self.function_declaration || other.function_declaration,
            variable_declaration: self.variable_declaration || other.variable_declaration,
            string_literal: self.string_literal || other.string_literal,
            object_expression: self.object_expression || other.object_expression,
            array_expression: self.array_expression || other.array_expression,
            debugger_statement: self.debugger_statement || other.debugger_statement,
        }
    }
}

/// A simplified AST node for the WASM boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WitAstNode {
    /// An import declaration.
    ImportDecl(ImportDeclarationNode),
    /// A `debugger` statement.
    DebuggerStmt(DebuggerStatementNode),
    /// A call expression.
    CallExpr(CallExpressionNode),
    /// An export default declaration.
    ExportDefaultDecl(ExportDefaultNode),
    /// An export named declaration.
    ExportNamedDecl(ExportNamedNode),
}

/// Simplified import declaration for WASM plugins.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDeclarationNode {
    /// Source span.
    pub span: Span,
    /// Import source module.
    pub source: String,
    /// Import specifiers.
    pub specifiers: Vec<ImportSpecifierNode>,
}

/// A single import specifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSpecifierNode {
    /// Local binding name.
    pub local: String,
    /// Imported name (may differ from local).
    pub imported: Option<String>,
}

/// Simplified debugger statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebuggerStatementNode {
    /// Source span.
    pub span: Span,
}

/// Simplified call expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallExpressionNode {
    /// Source span.
    pub span: Span,
    /// Callee as a dot-separated path (e.g. "console.log").
    pub callee_path: String,
    /// Number of arguments.
    pub argument_count: u32,
    /// Whether this call is awaited.
    pub is_awaited: bool,
}

/// Simplified export default declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportDefaultNode {
    /// Source span.
    pub span: Span,
}

/// Simplified export named declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportNamedNode {
    /// Source span.
    pub span: Span,
    /// Exported names.
    pub names: Vec<String>,
}

/// File context provided to WASM plugins.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContext {
    /// File path.
    pub file_path: String,
    /// Full source text.
    pub source_text: String,
    /// File extension.
    pub extension: String,
}

/// A batch of nodes for a single file, sent to a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeBatch {
    /// File context.
    pub file: FileContext,
    /// Collected nodes matching the plugin's interests.
    pub nodes: Vec<WitAstNode>,
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    #[test]
    fn test_node_interest_default_is_empty() {
        let interest = NodeInterest::default();
        assert!(!interest.any(), "default interest should have no flags set");
    }

    #[test]
    fn test_node_interest_union() {
        let a = NodeInterest {
            import_declaration: true,
            ..NodeInterest::default()
        };
        let b = NodeInterest {
            call_expression: true,
            ..NodeInterest::default()
        };
        let combined = a.union(b);
        assert!(combined.import_declaration, "union should include import_declaration");
        assert!(combined.call_expression, "union should include call_expression");
        assert!(!combined.debugger_statement, "union should not include debugger_statement");
    }

    #[test]
    fn test_node_batch_serialization() {
        let batch = NodeBatch {
            file: FileContext {
                file_path: "test.ts".to_owned(),
                source_text: "debugger;".to_owned(),
                extension: "ts".to_owned(),
            },
            nodes: vec![WitAstNode::DebuggerStmt(DebuggerStatementNode {
                span: Span::new(0, 9),
            })],
        };
        let json = serde_json::to_string(&batch);
        assert!(json.is_ok(), "node batch should serialize to JSON");
    }
}
