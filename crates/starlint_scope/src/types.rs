//! Core types for scope analysis.

use std::sync::Arc;

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
    /// The binding's name (reference-counted to share with scope bindings map).
    pub name: Arc<str>,
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

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    // ── SymbolId ──────────────────────────────────────────────────

    #[test]
    fn symbol_id_index_zero() {
        let id = SymbolId(0);
        assert_eq!(id.index(), 0, "SymbolId(0) should convert to index 0");
    }

    #[test]
    fn symbol_id_index_nonzero() {
        let id = SymbolId(42);
        assert_eq!(id.index(), 42, "SymbolId(42) should convert to index 42");
    }

    #[test]
    #[allow(clippy::as_conversions)]
    fn symbol_id_index_large_value() {
        let id = SymbolId(u32::MAX);
        assert_eq!(
            id.index(),
            u32::MAX as usize,
            "SymbolId(u32::MAX) should convert to usize without overflow"
        );
    }

    #[test]
    fn symbol_id_equality() {
        let a = SymbolId(7);
        let b = SymbolId(7);
        let c = SymbolId(8);
        assert_eq!(a, b, "SymbolId with same inner value should be equal");
        assert_ne!(
            a, c,
            "SymbolId with different inner value should not be equal"
        );
    }

    #[test]
    fn symbol_id_copy() {
        let original = SymbolId(10);
        let copied = original;
        assert_eq!(
            original, copied,
            "copied SymbolId should equal original (Copy semantics)"
        );
    }

    // ── ScopeId ───────────────────────────────────────────────────

    #[test]
    fn scope_id_index_zero() {
        let id = ScopeId(0);
        assert_eq!(id.index(), 0, "ScopeId(0) should convert to index 0");
    }

    #[test]
    fn scope_id_index_nonzero() {
        let id = ScopeId(5);
        assert_eq!(id.index(), 5, "ScopeId(5) should convert to index 5");
    }

    #[test]
    #[allow(clippy::as_conversions)]
    fn scope_id_index_large_value() {
        let id = ScopeId(u32::MAX);
        assert_eq!(
            id.index(),
            u32::MAX as usize,
            "ScopeId(u32::MAX) should convert to usize without overflow"
        );
    }

    #[test]
    fn scope_id_equality() {
        let a = ScopeId(3);
        let b = ScopeId(3);
        let c = ScopeId(4);
        assert_eq!(a, b, "ScopeId with same inner value should be equal");
        assert_ne!(
            a, c,
            "ScopeId with different inner value should not be equal"
        );
    }

    // ── SymbolFlags ───────────────────────────────────────────────

    #[test]
    fn symbol_flags_contains_self() {
        assert!(
            SymbolFlags::VAR.contains(SymbolFlags::VAR),
            "VAR should contain itself"
        );
        assert!(
            SymbolFlags::LET.contains(SymbolFlags::LET),
            "LET should contain itself"
        );
        assert!(
            SymbolFlags::CONST_VARIABLE.contains(SymbolFlags::CONST_VARIABLE),
            "CONST_VARIABLE should contain itself"
        );
        assert!(
            SymbolFlags::FUNCTION.contains(SymbolFlags::FUNCTION),
            "FUNCTION should contain itself"
        );
        assert!(
            SymbolFlags::CLASS.contains(SymbolFlags::CLASS),
            "CLASS should contain itself"
        );
        assert!(
            SymbolFlags::IMPORT.contains(SymbolFlags::IMPORT),
            "IMPORT should contain itself"
        );
        assert!(
            SymbolFlags::CATCH_VARIABLE.contains(SymbolFlags::CATCH_VARIABLE),
            "CATCH_VARIABLE should contain itself"
        );
        assert!(
            SymbolFlags::FUNCTION_PARAM.contains(SymbolFlags::FUNCTION_PARAM),
            "FUNCTION_PARAM should contain itself"
        );
        assert!(
            SymbolFlags::USING.contains(SymbolFlags::USING),
            "USING should contain itself"
        );
    }

    #[test]
    fn symbol_flags_does_not_contain_different_flag() {
        assert!(
            !SymbolFlags::VAR.contains(SymbolFlags::LET),
            "VAR should not contain LET"
        );
        assert!(
            !SymbolFlags::LET.contains(SymbolFlags::CONST_VARIABLE),
            "LET should not contain CONST_VARIABLE"
        );
        assert!(
            !SymbolFlags::FUNCTION.contains(SymbolFlags::CLASS),
            "FUNCTION should not contain CLASS"
        );
    }

    #[test]
    fn symbol_flags_union_combines_flags() {
        let combined = SymbolFlags::VAR.union(SymbolFlags::FUNCTION);
        assert!(
            combined.contains(SymbolFlags::VAR),
            "union of VAR and FUNCTION should contain VAR"
        );
        assert!(
            combined.contains(SymbolFlags::FUNCTION),
            "union of VAR and FUNCTION should contain FUNCTION"
        );
        assert!(
            !combined.contains(SymbolFlags::LET),
            "union of VAR and FUNCTION should not contain LET"
        );
    }

    #[test]
    fn symbol_flags_union_multiple() {
        let combined = SymbolFlags::LET
            .union(SymbolFlags::CLASS)
            .union(SymbolFlags::IMPORT);
        assert!(
            combined.contains(SymbolFlags::LET),
            "triple union should contain LET"
        );
        assert!(
            combined.contains(SymbolFlags::CLASS),
            "triple union should contain CLASS"
        );
        assert!(
            combined.contains(SymbolFlags::IMPORT),
            "triple union should contain IMPORT"
        );
        assert!(
            !combined.contains(SymbolFlags::VAR),
            "triple union should not contain VAR"
        );
    }

    #[test]
    fn symbol_flags_union_with_self_is_idempotent() {
        let doubled = SymbolFlags::VAR.union(SymbolFlags::VAR);
        assert_eq!(
            doubled,
            SymbolFlags::VAR,
            "union of a flag with itself should be the same flag"
        );
    }

    #[test]
    fn symbol_flags_is_hoistable_var() {
        assert!(SymbolFlags::VAR.is_hoistable(), "VAR should be hoistable");
    }

    #[test]
    fn symbol_flags_is_hoistable_function() {
        assert!(
            SymbolFlags::FUNCTION.is_hoistable(),
            "FUNCTION should be hoistable"
        );
    }

    #[test]
    fn symbol_flags_is_hoistable_combined_var_function() {
        let combined = SymbolFlags::VAR.union(SymbolFlags::FUNCTION);
        assert!(
            combined.is_hoistable(),
            "union of VAR and FUNCTION should be hoistable"
        );
    }

    #[test]
    fn symbol_flags_not_hoistable_let() {
        assert!(
            !SymbolFlags::LET.is_hoistable(),
            "LET should not be hoistable"
        );
    }

    #[test]
    fn symbol_flags_not_hoistable_const() {
        assert!(
            !SymbolFlags::CONST_VARIABLE.is_hoistable(),
            "CONST_VARIABLE should not be hoistable"
        );
    }

    #[test]
    fn symbol_flags_not_hoistable_class() {
        assert!(
            !SymbolFlags::CLASS.is_hoistable(),
            "CLASS should not be hoistable"
        );
    }

    #[test]
    fn symbol_flags_not_hoistable_import() {
        assert!(
            !SymbolFlags::IMPORT.is_hoistable(),
            "IMPORT should not be hoistable"
        );
    }

    #[test]
    fn symbol_flags_not_hoistable_catch_variable() {
        assert!(
            !SymbolFlags::CATCH_VARIABLE.is_hoistable(),
            "CATCH_VARIABLE should not be hoistable"
        );
    }

    #[test]
    fn symbol_flags_not_hoistable_function_param() {
        assert!(
            !SymbolFlags::FUNCTION_PARAM.is_hoistable(),
            "FUNCTION_PARAM should not be hoistable"
        );
    }

    #[test]
    fn symbol_flags_not_hoistable_using() {
        assert!(
            !SymbolFlags::USING.is_hoistable(),
            "USING should not be hoistable"
        );
    }

    #[test]
    fn symbol_flags_hoistable_in_union_with_non_hoistable() {
        let combined = SymbolFlags::VAR.union(SymbolFlags::CLASS);
        assert!(
            combined.is_hoistable(),
            "union containing VAR should be hoistable even when combined with CLASS"
        );
    }

    #[test]
    fn symbol_flags_all_values_are_distinct() {
        let all_flags = [
            SymbolFlags::VAR,
            SymbolFlags::LET,
            SymbolFlags::CONST_VARIABLE,
            SymbolFlags::FUNCTION,
            SymbolFlags::CLASS,
            SymbolFlags::IMPORT,
            SymbolFlags::CATCH_VARIABLE,
            SymbolFlags::FUNCTION_PARAM,
            SymbolFlags::USING,
        ];
        for (i, a) in all_flags.iter().enumerate() {
            for (j, b) in all_flags.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "SymbolFlags at index {i} and {j} should be distinct");
                    assert!(
                        !a.contains(*b),
                        "SymbolFlags at index {i} should not contain flag at index {j}"
                    );
                }
            }
        }
    }

    // ── ReferenceFlags ────────────────────────────────────────────

    #[test]
    fn reference_flags_read_is_read() {
        assert!(
            ReferenceFlags::READ.is_read(),
            "READ should report is_read() as true"
        );
    }

    #[test]
    fn reference_flags_read_is_not_write() {
        assert!(
            !ReferenceFlags::READ.is_write(),
            "READ should report is_write() as false"
        );
    }

    #[test]
    fn reference_flags_write_is_write() {
        assert!(
            ReferenceFlags::WRITE.is_write(),
            "WRITE should report is_write() as true"
        );
    }

    #[test]
    fn reference_flags_write_is_not_read() {
        assert!(
            !ReferenceFlags::WRITE.is_read(),
            "WRITE should report is_read() as false"
        );
    }

    #[test]
    fn reference_flags_read_write_is_both() {
        assert!(
            ReferenceFlags::READ_WRITE.is_read(),
            "READ_WRITE should report is_read() as true"
        );
        assert!(
            ReferenceFlags::READ_WRITE.is_write(),
            "READ_WRITE should report is_write() as true"
        );
    }

    #[test]
    fn reference_flags_read_write_equals_read_or_write() {
        let manual = ReferenceFlags(ReferenceFlags::READ.0 | ReferenceFlags::WRITE.0);
        assert_eq!(
            ReferenceFlags::READ_WRITE,
            manual,
            "READ_WRITE should equal the bitwise OR of READ and WRITE"
        );
    }

    #[test]
    fn reference_flags_read_and_write_are_distinct() {
        assert_ne!(
            ReferenceFlags::READ,
            ReferenceFlags::WRITE,
            "READ and WRITE should be distinct flags"
        );
    }

    #[test]
    fn reference_flags_read_write_differs_from_read_and_write() {
        assert_ne!(
            ReferenceFlags::READ_WRITE,
            ReferenceFlags::READ,
            "READ_WRITE should not equal READ alone"
        );
        assert_ne!(
            ReferenceFlags::READ_WRITE,
            ReferenceFlags::WRITE,
            "READ_WRITE should not equal WRITE alone"
        );
    }
}
