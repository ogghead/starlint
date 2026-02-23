//! Two-pass scope builder that produces [`ScopeData`] from an [`AstTree`].
//!
//! **Pass 1**: Walk the tree collecting declarations, building the scope tree,
//! and recording write targets (assignment/update/for-in/for-of).
//! **Pass 2**: Walk again resolving identifier references against the scope chain.

use std::collections::HashMap;
use std::sync::Arc;

use starlint_ast::node::AstNode;
use starlint_ast::operator::{AssignmentOperator, VariableDeclarationKind};
use starlint_ast::tree::AstTree;
use starlint_ast::types::{NodeId, Span};

use crate::scope_data::{ScopeData, ScopeInfo};
use crate::types::{
    ReferenceFlags, ReferenceInfo, ScopeId, SymbolFlags, SymbolId, SymbolInfo, UnresolvedRef,
};

/// Build scope analysis data from an `AstTree`.
#[must_use]
pub fn build(tree: &AstTree) -> ScopeData {
    let mut builder = ScopeBuilder::new(tree);
    builder.pass1_collect_declarations();
    builder.pass2_resolve_references();
    builder.finish()
}

/// Internal builder state.
struct ScopeBuilder<'a> {
    /// The AST to analyze.
    tree: &'a AstTree,
    /// All symbols collected so far.
    symbols: Vec<SymbolInfo>,
    /// All scopes collected so far.
    scopes: Vec<ScopeInfo>,
    /// Resolved references per symbol.
    resolved_refs: Vec<Vec<ReferenceInfo>>,
    /// Unresolved references by name.
    unresolved: HashMap<Arc<str>, Vec<UnresolvedRef>>,
    /// Span-to-symbol lookup.
    span_to_symbol: HashMap<(u32, u32), SymbolId>,
    /// Fast `NodeId` → `ScopeId` lookup (populated during scope creation).
    node_to_scope: HashMap<NodeId, ScopeId>,
    /// Write targets collected during pass 1 (avoids a separate traversal).
    write_targets: HashMap<NodeId, ReferenceFlags>,
}

impl<'a> ScopeBuilder<'a> {
    /// Create a new builder for the given tree.
    fn new(tree: &'a AstTree) -> Self {
        Self {
            tree,
            symbols: Vec::new(),
            scopes: Vec::new(),
            resolved_refs: Vec::new(),
            unresolved: HashMap::new(),
            span_to_symbol: HashMap::new(),
            node_to_scope: HashMap::new(),
            write_targets: HashMap::new(),
        }
    }

    /// Pass 1: Collect all declarations and build the scope tree.
    fn pass1_collect_declarations(&mut self) {
        // The scope stack tracks (ScopeId, span_end) — we pop when we pass the end.
        let mut scope_stack: Vec<(ScopeId, u32)> = Vec::new();
        // Function scope stack for var hoisting.
        let mut fn_scope_stack: Vec<ScopeId> = Vec::new();

        for (node_id, node) in self.tree.iter() {
            let node_span = node.span();

            // Pop scopes whose span has ended.
            while let Some(&(_, end)) = scope_stack.last() {
                if node_span.start >= end {
                    scope_stack.pop();
                    // Pop fn_scope_stack if this was a function scope.
                    if fn_scope_stack.len() > scope_stack.len().saturating_add(1) {
                        fn_scope_stack.pop();
                    }
                } else {
                    break;
                }
            }

            // Push scope for scope-creating nodes.
            if let Some(is_function_scope) = is_scope_creating(node) {
                let current_scope = scope_stack.last().map(|&(id, _)| id);
                let scope_id = self.push_scope(current_scope, node_id);
                scope_stack.push((scope_id, node_span.end));
                if is_function_scope {
                    fn_scope_stack.push(scope_id);
                } else if fn_scope_stack.is_empty() {
                    // Block scope inside root — fn_scope remains root.
                }
            }

            let current_scope = scope_stack.last().map(|&(id, _)| id);
            let fn_scope = fn_scope_stack.last().copied().or(current_scope);

            // Collect declarations.
            match node {
                AstNode::BindingIdentifier(binding) => {
                    // Determine flags from parent context.
                    let flags = self.determine_binding_flags(node_id);
                    let target_scope = if flags.contains(SymbolFlags::FUNCTION) {
                        // Function declaration names go to the ENCLOSING scope
                        // (not their own function scope). Use the second-to-last
                        // fn scope, or fall back to the root scope.
                        if fn_scope_stack.len() >= 2 {
                            fn_scope_stack
                                .get(fn_scope_stack.len().wrapping_sub(2))
                                .copied()
                        } else {
                            Some(ScopeId(0))
                        }
                    } else if flags.contains(SymbolFlags::VAR) {
                        // var declarations hoist to nearest function scope.
                        fn_scope
                    } else if flags.contains(SymbolFlags::IMPORT) {
                        // Imports go to root scope.
                        Some(ScopeId(0))
                    } else {
                        // let/const/class/catch/param go to current scope.
                        current_scope
                    };

                    if let Some(scope_id) = target_scope {
                        self.register_symbol(&binding.name, binding.span, scope_id, flags);
                    }
                }
                // Handle import specifiers — they declare bindings but don't use
                // BindingIdentifier nodes in the AST.
                AstNode::ImportSpecifier(spec) => {
                    // Only register if not already registered by BindingIdentifier.
                    let key = (spec.span.start, spec.span.end);
                    if !self.span_to_symbol.contains_key(&key) {
                        self.register_symbol(
                            &spec.local,
                            spec.span,
                            ScopeId(0), // imports always at root
                            SymbolFlags::IMPORT,
                        );
                    }
                }
                // Collect write targets in the same traversal (avoids a
                // separate pass over the tree).
                AstNode::AssignmentExpression(assign) => {
                    let flags = if assign.operator == AssignmentOperator::Assign {
                        ReferenceFlags::WRITE
                    } else {
                        ReferenceFlags::READ_WRITE
                    };
                    self.write_targets.insert(assign.left, flags);
                }
                AstNode::UpdateExpression(update) => {
                    self.write_targets
                        .insert(update.argument, ReferenceFlags::READ_WRITE);
                }
                AstNode::ForInStatement(f) => {
                    self.write_targets.insert(f.left, ReferenceFlags::WRITE);
                }
                AstNode::ForOfStatement(f) => {
                    self.write_targets.insert(f.left, ReferenceFlags::WRITE);
                }
                _ => {}
            }
        }
    }

    /// Pass 2: Resolve all identifier references.
    ///
    /// Write targets were already collected during pass 1, so this pass only
    /// needs to rebuild the scope stack and resolve references.
    #[allow(clippy::indexing_slicing)]
    fn pass2_resolve_references(&mut self) {
        let mut scope_stack: Vec<(ScopeId, u32)> = Vec::new();

        for (node_id, node) in self.tree.iter() {
            let node_span = node.span();

            // Pop scopes whose span has ended.
            while let Some(&(_, end)) = scope_stack.last() {
                if node_span.start >= end {
                    scope_stack.pop();
                } else {
                    break;
                }
            }

            // Push scope for scope-creating nodes.
            if is_scope_creating(node).is_some() {
                // Find the scope that was created for this node.
                if let Some(scope_id) = self.find_scope_by_node(node_id) {
                    scope_stack.push((scope_id, node_span.end));
                }
            }

            // Resolve identifier references.
            if let AstNode::IdentifierReference(ident) = node {
                let current_scope = scope_stack.last().map(|&(id, _)| id);
                let flags = self
                    .write_targets
                    .get(&node_id)
                    .copied()
                    .unwrap_or(ReferenceFlags::READ);

                if let Some(scope_id) = current_scope {
                    if let Some(symbol_id) = self.resolve_in_scope_chain(scope_id, &ident.name) {
                        // Resolved reference.
                        self.resolved_refs[symbol_id.index()].push(ReferenceInfo {
                            symbol_id,
                            span: ident.span,
                            flags,
                        });
                    } else {
                        // Unresolved reference.
                        self.unresolved
                            .entry(Arc::from(ident.name.as_str()))
                            .or_default()
                            .push(UnresolvedRef { span: ident.span });
                    }
                }
            }
        }
    }

    /// Push a new scope.
    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
    fn push_scope(&mut self, parent: Option<ScopeId>, node_id: NodeId) -> ScopeId {
        let id = ScopeId(self.scopes.len() as u32);
        self.scopes.push(ScopeInfo {
            parent,
            node_id,
            bindings: HashMap::new(),
        });
        self.node_to_scope.insert(node_id, id);
        id
    }

    /// Register a symbol in a scope.
    #[allow(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        clippy::indexing_slicing
    )]
    fn register_symbol(&mut self, name: &str, span: Span, scope_id: ScopeId, flags: SymbolFlags) {
        // Check for redeclaration in the same scope.
        if let Some(&existing_id) = self.scopes[scope_id.index()].bindings.get(name) {
            // Redeclaration — add to existing symbol's redeclarations list.
            self.symbols[existing_id.index()].redeclarations.push(span);
            // Also add a span-to-symbol mapping for the redeclaration.
            self.span_to_symbol
                .insert((span.start, span.end), existing_id);
            return;
        }

        let id = SymbolId(self.symbols.len() as u32);
        // Allocate the name string once and share via Arc between symbol and scope bindings.
        let name_arc: Arc<str> = Arc::from(name);
        self.symbols.push(SymbolInfo {
            name: Arc::clone(&name_arc),
            span,
            scope_id,
            flags,
            redeclarations: Vec::new(),
        });
        self.resolved_refs.push(Vec::new());
        self.scopes[scope_id.index()].bindings.insert(name_arc, id);
        self.span_to_symbol.insert((span.start, span.end), id);
    }

    /// Determine what kind of binding a `BindingIdentifier` represents.
    fn determine_binding_flags(&self, node_id: NodeId) -> SymbolFlags {
        // Walk up the parent chain to find the declaration context.
        let mut current = node_id;
        loop {
            let Some(parent_id) = self.tree.parent(current) else {
                return SymbolFlags::VAR; // fallback
            };
            let Some(parent) = self.tree.get(parent_id) else {
                return SymbolFlags::VAR;
            };

            match parent {
                AstNode::VariableDeclarator(_) => {
                    // Keep walking up to find VariableDeclaration for the kind.
                    current = parent_id;
                }
                AstNode::VariableDeclaration(decl) => {
                    return match decl.kind {
                        VariableDeclarationKind::Var => SymbolFlags::VAR,
                        VariableDeclarationKind::Let => SymbolFlags::LET,
                        VariableDeclarationKind::Const => SymbolFlags::CONST_VARIABLE,
                        VariableDeclarationKind::Using | VariableDeclarationKind::AwaitUsing => {
                            SymbolFlags::USING
                        }
                    };
                }
                AstNode::Function(f) => {
                    // Check if this binding is the function's name.
                    if f.id == Some(node_id) || f.id == Some(current) {
                        return SymbolFlags::FUNCTION;
                    }
                    // Otherwise it's a parameter.
                    return SymbolFlags::FUNCTION_PARAM;
                }
                AstNode::ArrowFunctionExpression(_) => {
                    return SymbolFlags::FUNCTION_PARAM;
                }
                AstNode::Class(c) => {
                    if c.id == Some(node_id) || c.id == Some(current) {
                        return SymbolFlags::CLASS;
                    }
                    return SymbolFlags::VAR; // shouldn't happen
                }
                AstNode::CatchClause(_) => {
                    return SymbolFlags::CATCH_VARIABLE;
                }
                AstNode::ImportDeclaration(_) | AstNode::ImportSpecifier(_) => {
                    return SymbolFlags::IMPORT;
                }
                // Patterns (destructuring, assignment, array, object, rest)
                // — keep walking up.
                AstNode::ArrayPattern(_)
                | AstNode::ObjectPattern(_)
                | AstNode::ObjectProperty(_)
                | AstNode::AssignmentPattern(_)
                | AstNode::SpreadElement(_) => {
                    current = parent_id;
                }
                _ => {
                    return SymbolFlags::VAR; // fallback
                }
            }
        }
    }

    /// Resolve a name by walking up the scope chain.
    #[allow(clippy::indexing_slicing)]
    fn resolve_in_scope_chain(&self, start_scope: ScopeId, name: &str) -> Option<SymbolId> {
        let mut scope_id = Some(start_scope);
        while let Some(sid) = scope_id {
            if let Some(&symbol_id) = self.scopes[sid.index()].bindings.get(name) {
                return Some(symbol_id);
            }
            scope_id = self.scopes[sid.index()].parent;
        }
        None
    }

    /// Find the scope created for a specific AST node (O(1) lookup).
    fn find_scope_by_node(&self, node_id: NodeId) -> Option<ScopeId> {
        self.node_to_scope.get(&node_id).copied()
    }

    /// Consume the builder and produce `ScopeData`.
    fn finish(self) -> ScopeData {
        ScopeData {
            symbols: self.symbols,
            scopes: self.scopes,
            resolved_refs: self.resolved_refs,
            unresolved: self.unresolved,
            span_to_symbol: self.span_to_symbol,
        }
    }
}

/// Returns `Some(is_function_scope)` if the node creates a new scope.
///
/// Function scopes receive hoisted `var` declarations.
const fn is_scope_creating(node: &AstNode) -> Option<bool> {
    match node {
        AstNode::Program(_) | AstNode::Function(_) | AstNode::ArrowFunctionExpression(_) => {
            Some(true)
        }
        AstNode::BlockStatement(_)
        | AstNode::ForStatement(_)
        | AstNode::ForInStatement(_)
        | AstNode::ForOfStatement(_)
        | AstNode::CatchClause(_)
        | AstNode::SwitchStatement(_)
        | AstNode::StaticBlock(_) => Some(false),
        _ => None,
    }
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::as_conversions,
    clippy::cast_possible_truncation
)]
mod tests {
    use starlint_parser::ParseOptions;

    use super::build;

    /// Parse JS source and build scope data.
    fn build_from_source(source: &str) -> super::ScopeData {
        let options = ParseOptions {
            jsx: false,
            typescript: false,
            module: true,
        };
        let result = starlint_parser::parse(source, options);
        build(&result.tree)
    }

    #[test]
    fn test_var_declaration() {
        let data = build_from_source("var x = 1;");
        assert_eq!(data.symbols.len(), 1);
        assert_eq!(&*data.symbols[0].name, "x");
        assert!(data.symbols[0].flags.contains(super::SymbolFlags::VAR));
    }

    #[test]
    fn test_let_const() {
        let data = build_from_source("let a = 1; const b = 2;");
        assert_eq!(data.symbols.len(), 2);
        assert!(data.symbols[0].flags.contains(super::SymbolFlags::LET));
        assert!(
            data.symbols[1]
                .flags
                .contains(super::SymbolFlags::CONST_VARIABLE)
        );
    }

    #[test]
    fn test_function_declaration() {
        let data = build_from_source("function foo() {}");
        let foo = data.symbols.iter().find(|s| *s.name == *"foo");
        assert!(foo.is_some(), "should have symbol 'foo'");
        assert!(
            foo.unwrap().flags.contains(super::SymbolFlags::FUNCTION),
            "should be FUNCTION"
        );
    }

    #[test]
    fn test_reference_resolution() {
        let data = build_from_source("const x = 1; console.log(x);");
        let x_sym = data.symbol_by_span(data.symbols[0].span);
        assert!(x_sym.is_some());
        let refs = data.get_resolved_references(x_sym.unwrap());
        assert_eq!(refs.len(), 1, "x should have 1 reference");
        assert!(refs[0].flags.is_read(), "should be a read reference");
    }

    #[test]
    fn test_write_reference() {
        let data = build_from_source("let x = 1; x = 2;");
        let x_sym = data.symbols.iter().position(|s| *s.name == *"x").unwrap();
        let x_id = super::SymbolId(x_sym as u32);
        let refs = data.get_resolved_references(x_id);
        assert_eq!(refs.len(), 1, "x should have 1 reference (the assignment)");
        assert!(refs[0].flags.is_write(), "should be a write reference");
    }

    #[test]
    fn test_unresolved_reference() {
        let data = build_from_source("console.log(x);");
        assert!(
            data.unresolved.contains_key("console"),
            "console should be unresolved"
        );
    }

    #[test]
    fn test_var_hoisting() {
        let data = build_from_source("function foo() { if (true) { var x = 1; } }");
        let x = data.symbols.iter().find(|s| *s.name == *"x").unwrap();
        let x_scope = x.scope_id;
        assert_ne!(x_scope, super::ScopeId(0), "x should not be in root scope");
    }

    #[test]
    fn test_scope_chain_resolution() {
        let data = build_from_source("const x = 1; function foo() { return x; }");
        let x_sym = data.symbols.iter().position(|s| *s.name == *"x").unwrap();
        let x_id = super::SymbolId(x_sym as u32);
        let refs = data.get_resolved_references(x_id);
        assert_eq!(refs.len(), 1, "x should be referenced from inside foo");
    }

    #[test]
    fn test_shadowing() {
        let data = build_from_source("const x = 1; function foo() { const x = 2; }");
        let x_symbols: Vec<_> = data.symbols.iter().filter(|s| *s.name == *"x").collect();
        assert_eq!(x_symbols.len(), 2, "should have 2 symbols named x");
        assert_ne!(
            x_symbols[0].scope_id, x_symbols[1].scope_id,
            "should be in different scopes"
        );
    }

    #[test]
    fn test_import_binding() {
        let data = build_from_source("import { foo } from 'bar'; foo();");
        let foo = data.symbols.iter().find(|s| *s.name == *"foo");
        assert!(foo.is_some(), "should have import symbol");
        assert!(
            foo.unwrap().flags.contains(super::SymbolFlags::IMPORT),
            "should be IMPORT"
        );
    }

    #[test]
    fn test_redeclaration() {
        let data = build_from_source("var x = 1; var x = 2;");
        let x = data.symbols.iter().find(|s| *s.name == *"x").unwrap();
        assert_eq!(x.redeclarations.len(), 1, "should have 1 redeclaration");
    }

    #[test]
    fn test_compound_assignment() {
        let data = build_from_source("let x = 1; x += 2;");
        let x_sym = data.symbols.iter().position(|s| *s.name == *"x").unwrap();
        let x_id = super::SymbolId(x_sym as u32);
        let refs = data.get_resolved_references(x_id);
        assert_eq!(refs.len(), 1);
        assert!(refs[0].flags.is_read(), "compound assignment reads");
        assert!(refs[0].flags.is_write(), "compound assignment writes");
    }

    #[test]
    fn test_catch_clause_param() {
        let data = build_from_source("try {} catch (e) { console.log(e); }");
        let e = data.symbols.iter().find(|s| *s.name == *"e");
        assert!(e.is_some(), "should have catch param");
        assert!(
            e.unwrap()
                .flags
                .contains(super::SymbolFlags::CATCH_VARIABLE),
            "should be CATCH_VARIABLE"
        );
    }

    #[test]
    fn test_function_params() {
        let data = build_from_source("function foo(a, b) { return a + b; }");
        let a = data.symbols.iter().find(|s| *s.name == *"a");
        assert!(a.is_some());
        assert!(
            a.unwrap()
                .flags
                .contains(super::SymbolFlags::FUNCTION_PARAM),
            "should be FUNCTION_PARAM"
        );
    }

    #[test]
    fn test_rename_symbol_edits() {
        use starlint_plugin_sdk::diagnostic::Span as DiagSpan;

        let data = build_from_source("const btn = 1; console.log(btn);");
        let btn_sym = data.symbols.iter().position(|s| *s.name == *"btn").unwrap();
        let btn_id = super::SymbolId(btn_sym as u32);
        let decl_span = data.symbol_span(btn_id);
        let edits = data.rename_symbol_edits(
            btn_id,
            "button",
            DiagSpan::new(decl_span.start, decl_span.end),
        );
        assert_eq!(
            edits.len(),
            2,
            "should have edit for declaration + 1 reference"
        );
    }

    #[test]
    fn test_function_write_reference() {
        let data = build_from_source("function foo() {} foo = bar;");
        let foo = data.symbols.iter().find(|s| *s.name == *"foo").unwrap();
        assert!(
            foo.flags.contains(super::SymbolFlags::FUNCTION),
            "foo should have FUNCTION flag, got {:?}",
            foo.flags
        );
        let foo_id = data.symbol_by_span(foo.span).unwrap();
        let refs = data.get_resolved_references(foo_id);
        assert_eq!(
            refs.len(),
            1,
            "foo should have 1 reference (the assignment)"
        );
        assert!(refs[0].flags.is_write(), "should be a write reference");
    }
}
