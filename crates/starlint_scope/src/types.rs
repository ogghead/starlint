//! Core types for scope analysis.

use starlint_ast::types::Span;

/// Index into the symbol table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub(crate) u32);

/// Index into the scope list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(pub(crate) u32);

impl SymbolId {
    /// Convert to `usize` for indexing.
    #[must_use]
    #[allow(clippy::as_conversions)]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

impl ScopeId {
    /// Convert to `usize` for indexing.
    #[must_use]
    #[allow(clippy::as_conversions)]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// Flags describing what kind of declaration a symbol is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolFlags(u16);

impl SymbolFlags {
    /// `var` declaration.
    pub const VAR: Self = Self(1 << 0);
    /// `let` declaration.
    pub const LET: Self = Self(1 << 1);
    /// `const` declaration.
    pub const CONST_VARIABLE: Self = Self(1 << 2);
    /// `function` declaration.
    pub const FUNCTION: Self = Self(1 << 3);
    /// `class` declaration.
    pub const CLASS: Self = Self(1 << 4);
    /// `import` binding.
    pub const IMPORT: Self = Self(1 << 5);
    /// `catch` clause parameter.
    pub const CATCH_VARIABLE: Self = Self(1 << 6);
    /// Function parameter.
    pub const FUNCTION_PARAM: Self = Self(1 << 7);
    /// `using` declaration.
    pub const USING: Self = Self(1 << 8);

    /// Check if this flag set contains a specific flag.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Combine two flag sets.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Whether the symbol is a `var` or function (hoistable).
    #[must_use]
    pub const fn is_hoistable(self) -> bool {
        self.contains(Self::VAR) || self.contains(Self::FUNCTION)
    }
}

/// Flags describing how a reference is used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReferenceFlags(u8);

impl ReferenceFlags {
    /// Read reference.
    pub const READ: Self = Self(1 << 0);
    /// Write reference.
    pub const WRITE: Self = Self(1 << 1);
    /// Both read and write (e.g., `x += 1`).
    pub const READ_WRITE: Self = Self(Self::READ.0 | Self::WRITE.0);

    /// Check if this is a read reference.
    #[must_use]
    pub const fn is_read(self) -> bool {
        (self.0 & Self::READ.0) != 0
    }

    /// Check if this is a write reference.
    #[must_use]
    pub const fn is_write(self) -> bool {
        (self.0 & Self::WRITE.0) != 0
    }
}

/// Information about a single symbol (declared binding).
#[derive(Debug)]
pub struct SymbolInfo {
    /// The binding's name.
    pub name: String,
    /// Span of the declaration site.
    pub span: Span,
    /// Which scope this symbol belongs to.
    pub scope_id: ScopeId,
    /// What kind of declaration.
    pub flags: SymbolFlags,
    /// Spans of redeclarations (for `no-redeclare`).
    pub redeclarations: Vec<Span>,
}

/// Information about a resolved reference.
#[derive(Debug)]
pub struct ReferenceInfo {
    /// Which symbol this reference resolves to.
    pub symbol_id: SymbolId,
    /// Span of the reference.
    pub span: Span,
    /// Read/write classification.
    pub flags: ReferenceFlags,
}

/// An unresolved reference (no matching declaration found).
#[derive(Debug)]
pub struct UnresolvedRef {
    /// Span of the unresolved reference.
    pub span: Span,
}
