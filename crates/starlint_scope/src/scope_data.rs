//! The main query struct for scope analysis results.

use std::collections::HashMap;

use starlint_ast::types::{NodeId, Span};
use starlint_plugin_sdk::diagnostic::Span as DiagSpan;

use crate::types::{ReferenceInfo, ScopeId, SymbolFlags, SymbolId, SymbolInfo, UnresolvedRef};

/// Scope information stored per scope.
#[derive(Debug)]
pub struct ScopeInfo {
    /// Parent scope, if any.
    pub parent: Option<ScopeId>,
    /// AST node that created this scope.
    pub node_id: NodeId,
    /// Bindings declared in this scope (name → symbol ID).
    pub bindings: HashMap<String, SymbolId>,
}

/// Result of scope analysis on an `AstTree`.
///
/// Provides symbol table, scope tree, reference tracking, and unresolved
/// reference detection — the same capabilities rules previously got from
/// `oxc_semantic::Semantic`.
#[derive(Debug)]
pub struct ScopeData {
    /// All symbols (declarations), indexed by `SymbolId`.
    pub(crate) symbols: Vec<SymbolInfo>,
    /// All scopes, indexed by `ScopeId`.
    pub(crate) scopes: Vec<ScopeInfo>,
    /// Resolved references per symbol, indexed by `SymbolId`.
    pub(crate) resolved_refs: Vec<Vec<ReferenceInfo>>,
    /// Unresolved references by name.
    pub(crate) unresolved: HashMap<String, Vec<UnresolvedRef>>,
    /// Span-to-symbol lookup for `symbol_by_span`.
    pub(crate) span_to_symbol: HashMap<(u32, u32), SymbolId>,
}

impl ScopeData {
    /// Find a symbol by its declaration span.
    #[must_use]
    pub fn symbol_by_span(&self, span: Span) -> Option<SymbolId> {
        self.span_to_symbol.get(&(span.start, span.end)).copied()
    }

    /// Get the flags for a symbol.
    ///
    /// # Panics
    ///
    /// Panics if `id` is out of bounds.
    #[must_use]
    #[allow(clippy::indexing_slicing)]
    pub fn symbol_flags(&self, id: SymbolId) -> SymbolFlags {
        self.symbols[id.index()].flags
    }

    /// Get the declaration span for a symbol.
    ///
    /// # Panics
    ///
    /// Panics if `id` is out of bounds.
    #[must_use]
    #[allow(clippy::indexing_slicing)]
    pub fn symbol_span(&self, id: SymbolId) -> Span {
        self.symbols[id.index()].span
    }

    /// Get the name of a symbol.
    ///
    /// # Panics
    ///
    /// Panics if `id` is out of bounds.
    #[must_use]
    #[allow(clippy::indexing_slicing)]
    pub fn symbol_name(&self, id: SymbolId) -> &str {
        &self.symbols[id.index()].name
    }

    /// Get the scope that a symbol belongs to.
    ///
    /// # Panics
    ///
    /// Panics if `id` is out of bounds.
    #[must_use]
    #[allow(clippy::indexing_slicing)]
    pub fn symbol_scope_id(&self, id: SymbolId) -> ScopeId {
        self.symbols[id.index()].scope_id
    }

    /// Get redeclaration spans for a symbol.
    ///
    /// # Panics
    ///
    /// Panics if `id` is out of bounds.
    #[must_use]
    #[allow(clippy::indexing_slicing)]
    pub fn symbol_redeclarations(&self, id: SymbolId) -> &[Span] {
        &self.symbols[id.index()].redeclarations
    }

    /// Iterate over all symbol IDs.
    pub fn symbol_ids(&self) -> impl Iterator<Item = SymbolId> + '_ {
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        (0..self.symbols.len()).map(|i| SymbolId(i as u32))
    }

    /// Get all resolved references to a symbol.
    ///
    /// # Panics
    ///
    /// Panics if `id` is out of bounds.
    #[must_use]
    #[allow(clippy::indexing_slicing)]
    pub fn get_resolved_references(&self, id: SymbolId) -> &[ReferenceInfo] {
        &self.resolved_refs[id.index()]
    }

    /// Get the root scope ID (always `ScopeId(0)`).
    #[must_use]
    pub const fn root_scope_id(&self) -> ScopeId {
        ScopeId(0)
    }

    /// Get the parent scope of a scope.
    ///
    /// # Panics
    ///
    /// Panics if `id` is out of bounds.
    #[must_use]
    #[allow(clippy::indexing_slicing)]
    pub fn scope_parent_id(&self, id: ScopeId) -> Option<ScopeId> {
        self.scopes[id.index()].parent
    }

    /// Look up a binding by name in a specific scope.
    ///
    /// # Panics
    ///
    /// Panics if `scope` is out of bounds.
    #[must_use]
    #[allow(clippy::indexing_slicing)]
    pub fn get_binding(&self, scope: ScopeId, name: &str) -> Option<SymbolId> {
        self.scopes[scope.index()].bindings.get(name).copied()
    }

    /// Get the map of unresolved references.
    #[must_use]
    pub const fn root_unresolved_references(&self) -> &HashMap<String, Vec<UnresolvedRef>> {
        &self.unresolved
    }

    /// Generate edits to rename a symbol (declaration + all references).
    ///
    /// Returns one edit for the declaration site and one for each reference.
    #[must_use]
    #[allow(clippy::indexing_slicing)]
    pub fn rename_symbol_edits(
        &self,
        symbol_id: SymbolId,
        new_name: &str,
        decl_span: DiagSpan,
    ) -> Vec<starlint_plugin_sdk::diagnostic::Edit> {
        use starlint_plugin_sdk::diagnostic::Edit;

        let mut edits = Vec::new();

        // Edit for the declaration site.
        edits.push(Edit {
            span: decl_span,
            replacement: new_name.to_owned(),
        });

        // Edit for each resolved reference.
        for reference in &self.resolved_refs[symbol_id.index()] {
            edits.push(Edit {
                span: DiagSpan::new(reference.span.start, reference.span.end),
                replacement: new_name.to_owned(),
            });
        }

        edits
    }
}
