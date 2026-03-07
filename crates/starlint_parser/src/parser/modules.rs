//! Module (import/export) parsing.

use starlint_ast::node::{
    AstNode, ExportAllDeclarationNode, ExportDefaultDeclarationNode, ExportNamedDeclarationNode,
    ExportSpecifierNode, ImportDeclarationNode, ImportSpecifierNode,
};
use starlint_ast::types::{NodeId, Span};

use crate::token::TokenKind;

use super::Parser;

impl Parser<'_> {
    /// Parse an import declaration.
    pub(crate) fn parse_import_declaration(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let import_id = self.reserve(parent);
        self.bump(); // `import`

        // `import type` (TS)
        let is_type_only =
            self.options.typescript && self.at(TokenKind::Type) && !self.peek_is_from_or_comma();
        if is_type_only {
            self.bump(); // `type`
        }

        let mut specifiers = Vec::new();

        // Side-effect import: `import "module"`
        if self.at(TokenKind::String) {
            let source = self.parse_string_value();
            self.expect_semicolon();
            self.tree.set(
                import_id,
                AstNode::ImportDeclaration(ImportDeclarationNode {
                    span: Span::new(start, self.prev_end),
                    source,
                    source_span: Span::EMPTY,
                    specifiers: Box::new([]),
                    import_kind_is_type: is_type_only,
                }),
            );
            return import_id;
        }

        // Default import: `import foo from "module"`
        if self.at(TokenKind::Identifier) || self.cur().is_keyword() {
            let local_start = self.start();
            let local_name = self.cur_text().to_owned();
            let local_tok = self.bump();
            let spec_id = self.push(
                AstNode::ImportSpecifier(ImportSpecifierNode {
                    span: Span::new(local_start, local_tok.end),
                    local: local_name,
                    imported: "default".to_owned(),
                    is_type: false,
                }),
                Some(import_id),
            );
            specifiers.push(spec_id);

            if self.eat(TokenKind::Comma) {
                // `import foo, { bar } from "module"` or `import foo, * as ns from "module"`
                if self.at(TokenKind::Star) {
                    // Namespace import
                    self.parse_namespace_import(import_id, &mut specifiers);
                } else if self.at(TokenKind::LBrace) {
                    self.parse_named_imports(import_id, &mut specifiers);
                }
            }
        } else if self.at(TokenKind::Star) {
            // Namespace import: `import * as ns from "module"`
            self.parse_namespace_import(import_id, &mut specifiers);
        } else if self.at(TokenKind::LBrace) {
            // Named imports: `import { foo, bar } from "module"`
            self.parse_named_imports(import_id, &mut specifiers);
        }

        // `from "module"`
        let _ = self.expect(TokenKind::From);
        let source = self.parse_string_value();
        self.expect_semicolon();

        self.tree.set(
            import_id,
            AstNode::ImportDeclaration(ImportDeclarationNode {
                span: Span::new(start, self.prev_end),
                source,
                source_span: Span::EMPTY,
                specifiers: specifiers.into_boxed_slice(),
                import_kind_is_type: is_type_only,
            }),
        );
        import_id
    }

    /// Parse namespace import `* as name`.
    fn parse_namespace_import(&mut self, parent: NodeId, specifiers: &mut Vec<NodeId>) {
        let ns_start = self.start();
        self.bump(); // `*`
        let _ = self.expect(TokenKind::As);
        let local_name = self.cur_text().to_owned();
        let local_tok = self.bump();
        let spec_id = self.push(
            AstNode::ImportSpecifier(ImportSpecifierNode {
                span: Span::new(ns_start, local_tok.end),
                local: local_name,
                imported: "*".to_owned(),
                is_type: false,
            }),
            Some(parent),
        );
        specifiers.push(spec_id);
    }

    /// Parse named imports `{ foo, bar as baz }`.
    fn parse_named_imports(&mut self, parent: NodeId, specifiers: &mut Vec<NodeId>) {
        self.bump(); // `{`
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            let spec_start = self.start();

            // Check for inline `type` keyword: `import { type Foo }`
            let is_type = if self.at(TokenKind::Type) {
                let next = self.peek_next_text();
                // `type` followed by an identifier (not `,` or `}`) is an inline type import
                if next == "other" {
                    self.bump(); // consume `type`
                    true
                } else {
                    false
                }
            } else {
                false
            };

            let imported_name = self.cur_text().to_owned();
            self.bump();

            let local_name = if self.eat(TokenKind::As) {
                let name = self.cur_text().to_owned();
                self.bump();
                name
            } else {
                imported_name.clone()
            };

            let spec_id = self.push(
                AstNode::ImportSpecifier(ImportSpecifierNode {
                    span: Span::new(spec_start, self.prev_end),
                    local: local_name,
                    imported: imported_name,
                    is_type,
                }),
                Some(parent),
            );
            specifiers.push(spec_id);

            if !self.at(TokenKind::RBrace) {
                self.eat(TokenKind::Comma);
            }
        }
        let _ = self.expect(TokenKind::RBrace);
    }

    /// Parse an export declaration.
    pub(crate) fn parse_export_declaration(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        self.bump(); // `export`

        // `export default ...`
        if self.at(TokenKind::Default) {
            return self.parse_export_default(start, parent);
        }

        // `export * from "module"`
        if self.at(TokenKind::Star) {
            return self.parse_export_all(start, parent);
        }

        // `export { ... }`
        if self.at(TokenKind::LBrace) {
            return self.parse_export_named(start, parent);
        }

        // `export type` (TS)
        if self.options.typescript && self.at(TokenKind::Type) {
            // Could be `export type { ... }` or `export type Foo = ...`
            // Check what follows
        }

        // `export var/let/const/function/class ...`
        let export_id = self.reserve(parent);
        let declaration = Some(self.parse_statement_with_parent(Some(export_id)));
        let end = declaration
            .and_then(|id| self.tree.span(id))
            .map_or(self.prev_end, |s| s.end);

        self.tree.set(
            export_id,
            AstNode::ExportNamedDeclaration(ExportNamedDeclarationNode {
                span: Span::new(start, end),
                declaration,
                specifiers: Box::new([]),
                source: None,
            }),
        );
        export_id
    }

    /// Parse `export default ...`.
    fn parse_export_default(&mut self, start: u32, parent: Option<NodeId>) -> NodeId {
        let export_id = self.reserve(parent);
        self.bump(); // `default`

        let declaration = match self.cur() {
            TokenKind::Function => self.parse_function_declaration(Some(export_id)),
            TokenKind::Class => self.parse_class_declaration(Some(export_id)),
            TokenKind::Async if self.peek_next_is_function() => {
                self.bump(); // `async`
                self.parse_function(Some(export_id), true)
            }
            _ => {
                let expr = self.parse_assignment_expression(Some(export_id));
                self.expect_semicolon();
                expr
            }
        };

        let end = self.tree.span(declaration).map_or(self.prev_end, |s| s.end);

        self.tree.set(
            export_id,
            AstNode::ExportDefaultDeclaration(ExportDefaultDeclarationNode {
                span: Span::new(start, end),
                declaration,
            }),
        );
        export_id
    }

    /// Parse `export * from "module"`.
    fn parse_export_all(&mut self, start: u32, parent: Option<NodeId>) -> NodeId {
        let export_id = self.reserve(parent);
        self.bump(); // `*`

        // Optional `as name`
        let exported = self.eat(TokenKind::As).then(|| {
            let name = self.cur_text().to_owned();
            self.bump();
            name
        });

        let _ = self.expect(TokenKind::From);
        let source = self.parse_string_value();
        self.expect_semicolon();

        self.tree.set(
            export_id,
            AstNode::ExportAllDeclaration(ExportAllDeclarationNode {
                span: Span::new(start, self.prev_end),
                source,
                exported,
            }),
        );
        export_id
    }

    /// Parse `export { ... } [from "module"]`.
    fn parse_export_named(&mut self, start: u32, parent: Option<NodeId>) -> NodeId {
        let export_id = self.reserve(parent);
        self.bump(); // `{`

        let mut specifiers = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            let spec_start = self.start();
            let local_name = self.cur_text().to_owned();
            self.bump();

            let exported_name = if self.eat(TokenKind::As) {
                let name = self.cur_text().to_owned();
                self.bump();
                name
            } else {
                local_name.clone()
            };

            let spec_id = self.push(
                AstNode::ExportSpecifier(ExportSpecifierNode {
                    span: Span::new(spec_start, self.prev_end),
                    local: local_name,
                    exported: exported_name,
                }),
                Some(export_id),
            );
            specifiers.push(spec_id);

            if !self.at(TokenKind::RBrace) {
                self.eat(TokenKind::Comma);
            }
        }
        let _ = self.expect(TokenKind::RBrace);

        // Optional `from "module"` (re-export)
        let source = self.at(TokenKind::From).then(|| {
            self.bump();
            self.parse_string_value()
        });
        self.expect_semicolon();

        self.tree.set(
            export_id,
            AstNode::ExportNamedDeclaration(ExportNamedDeclarationNode {
                span: Span::new(start, self.prev_end),
                declaration: None,
                specifiers: specifiers.into_boxed_slice(),
                source,
            }),
        );
        export_id
    }

    /// Parse a string literal and return its value (stripping quotes).
    fn parse_string_value(&mut self) -> String {
        if self.at(TokenKind::String) {
            let text = self.cur_text();
            let value = if text.len() >= 2 {
                text.get(1..text.len().saturating_sub(1))
                    .unwrap_or_default()
                    .to_owned()
            } else {
                String::new()
            };
            self.bump();
            value
        } else {
            self.error("expected string literal");
            String::new()
        }
    }

    /// Check if current position is `type` followed by `from` or `,`.
    fn peek_is_from_or_comma(&self) -> bool {
        #[allow(clippy::as_conversions)]
        let after = self
            .source
            .get(self.current.end as usize..)
            .unwrap_or_default()
            .trim_start();
        after.starts_with("from") || after.starts_with(',')
    }
}
