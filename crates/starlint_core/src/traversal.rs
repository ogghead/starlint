//! Single-pass AST traversal with interest-based rule dispatch.
//!
//! Traverses the oxc AST once, dispatching each node **only** to rules that
//! declared interest in that [`AstType`] via [`NativeRule::run_on_kinds`].
//! Rules that return `None` (the default) receive all nodes for backwards
//! compatibility.

use std::path::Path;

use oxc_ast::AstKind;
use oxc_ast::ast::Program;
use oxc_ast::ast_kind::{AST_TYPE_MAX, AstType};
use oxc_ast_visit::Visit;
use oxc_semantic::Semantic;

use crate::rule::{NativeLintContext, NativeRule};
use starlint_plugin_sdk::diagnostic::Diagnostic;

/// Traverse the AST and dispatch to all active rules.
///
/// Returns diagnostics from all rules combined.
/// For rules that need semantic analysis, use [`traverse_and_lint_with_semantic`].
pub fn traverse_and_lint<'a>(
    program: &Program<'a>,
    rules: &[Box<dyn NativeRule>],
    source_text: &'a str,
    file_path: &'a Path,
) -> Vec<Diagnostic> {
    traverse_and_lint_with_semantic(program, rules, source_text, file_path, None)
}

/// Traverse the AST and dispatch to all active rules, with optional semantic data.
///
/// When `semantic` is `Some`, rules can access scope/symbol information via
/// [`NativeLintContext::semantic()`].
pub fn traverse_and_lint_with_semantic<'a>(
    program: &Program<'a>,
    rules: &[Box<dyn NativeRule>],
    source_text: &'a str,
    file_path: &'a Path,
    semantic: Option<&'a Semantic<'a>>,
) -> Vec<Diagnostic> {
    let all_rules: Vec<&dyn NativeRule> = rules.iter().map(std::convert::AsRef::as_ref).collect();

    let traversal_rules: Vec<&dyn NativeRule> = all_rules
        .iter()
        .copied()
        .filter(|r| r.needs_traversal())
        .collect();

    let mut all_diagnostics = Vec::new();

    // Run traversal-based rules via single-pass visitor with dispatch table.
    if !traversal_rules.is_empty() {
        let table = DispatchTable::build(&traversal_rules);
        let ctx = match semantic {
            Some(sem) => NativeLintContext::with_semantic(source_text, file_path, sem),
            None => NativeLintContext::new(source_text, file_path),
        };
        let mut visitor = RuleDispatchVisitor {
            rules: &traversal_rules,
            table,
            ctx,
        };
        visitor.visit_program(program);
        all_diagnostics.extend(visitor.ctx.into_diagnostics());
    }

    // Run run_once for all rules (shared context, single pass).
    let mut run_once_ctx = match semantic {
        Some(sem) => NativeLintContext::with_semantic(source_text, file_path, sem),
        None => NativeLintContext::new(source_text, file_path),
    };
    for rule in &all_rules {
        rule.run_once(&mut run_once_ctx);
    }
    all_diagnostics.extend(run_once_ctx.into_diagnostics());

    all_diagnostics
}

// ---------------------------------------------------------------------------
// Dispatch table
// ---------------------------------------------------------------------------

/// Convert an `AstType` to a `usize` index for the dispatch table.
///
/// `AstType` is `#[repr(u8)]`, so the cast is lossless.
#[allow(clippy::as_conversions)]
#[inline]
fn ast_type_index(ty: AstType) -> usize {
    usize::from(ty as u8)
}

/// Maps [`AstType`] discriminants to the rule indices that handle them.
///
/// Built once from rule interest declarations. Rules that return
/// `None` from `run_on_kinds()` / `leave_on_kinds()` go into the wildcard
/// lists and receive every node.
///
/// Indices stored in the table refer to positions in the **original** rules
/// slice (e.g. `LintSession::native_rules`), so the visitor can look up
/// rules by index without an intermediate filtered vec.
pub(crate) struct DispatchTable {
    /// Per-AstType rule indices for `enter_node`. Index = `AstType as usize`.
    enter: Vec<Vec<usize>>,
    /// Per-AstType rule indices for `leave_node`.
    leave: Vec<Vec<usize>>,
    /// Rules that receive ALL nodes on enter (wildcard / backwards compat).
    enter_all: Vec<usize>,
    /// Rules that receive ALL nodes on leave.
    leave_all: Vec<usize>,
}

impl DispatchTable {
    /// Build the dispatch table from a subset of rules identified by index.
    ///
    /// `rules` is the full rule set (e.g. `LintSession::native_rules`).
    /// `traversal_indices` lists the indices of rules that need AST traversal.
    /// The stored indices point into `rules`, so the visitor can look up rules
    /// directly without an intermediate filtered vec.
    pub(crate) fn build_from_indices(
        rules: &[Box<dyn NativeRule>],
        traversal_indices: &[usize],
    ) -> Self {
        let size = usize::from(AST_TYPE_MAX).saturating_add(1);
        let mut table = Self {
            enter: vec![Vec::new(); size],
            leave: vec![Vec::new(); size],
            enter_all: Vec::new(),
            leave_all: Vec::new(),
        };

        for &idx in traversal_indices {
            let Some(rule) = rules.get(idx) else {
                continue;
            };

            match rule.run_on_kinds() {
                Some(kinds) => {
                    for &kind in kinds {
                        let slot = ast_type_index(kind);
                        if let Some(entry) = table.enter.get_mut(slot) {
                            entry.push(idx);
                        }
                    }
                }
                None => table.enter_all.push(idx),
            }

            match rule.leave_on_kinds() {
                Some(kinds) => {
                    for &kind in kinds {
                        let slot = ast_type_index(kind);
                        if let Some(entry) = table.leave.get_mut(slot) {
                            entry.push(idx);
                        }
                    }
                }
                None => table.leave_all.push(idx),
            }
        }

        table
    }

    /// Build the dispatch table from a filtered slice of rule refs.
    ///
    /// Used by `traverse_and_lint_with_semantic` (backwards compat for tests).
    /// Indices stored refer to positions in the `rules` slice.
    fn build(rules: &[&dyn NativeRule]) -> Self {
        let size = usize::from(AST_TYPE_MAX).saturating_add(1);
        let mut table = Self {
            enter: vec![Vec::new(); size],
            leave: vec![Vec::new(); size],
            enter_all: Vec::new(),
            leave_all: Vec::new(),
        };

        for (idx, rule) in rules.iter().enumerate() {
            match rule.run_on_kinds() {
                Some(kinds) => {
                    for &kind in kinds {
                        let slot = ast_type_index(kind);
                        if let Some(entry) = table.enter.get_mut(slot) {
                            entry.push(idx);
                        }
                    }
                }
                None => table.enter_all.push(idx),
            }

            match rule.leave_on_kinds() {
                Some(kinds) => {
                    for &kind in kinds {
                        let slot = ast_type_index(kind);
                        if let Some(entry) = table.leave.get_mut(slot) {
                            entry.push(idx);
                        }
                    }
                }
                None => table.leave_all.push(idx),
            }
        }

        table
    }

    /// Build a compact per-file table from the session table + active mask.
    ///
    /// Resolves indices to direct `&dyn NativeRule` references and excludes
    /// inactive rules. The resulting table needs no bounds checks or active
    /// mask lookups in the per-node hot loop.
    fn compact<'s>(&self, rules: &'s [Box<dyn NativeRule>], active: &[bool]) -> CompactTable<'s> {
        // Pre-resolve the wildcard lists once (they're merged into every slot).
        let enter_all_resolved: Vec<&'s dyn NativeRule> = self
            .enter_all
            .iter()
            .filter(|&&idx| active.get(idx).copied().unwrap_or(false))
            .filter_map(|&idx| rules.get(idx).map(AsRef::as_ref))
            .collect();
        let leave_all_resolved: Vec<&'s dyn NativeRule> = self
            .leave_all
            .iter()
            .filter(|&&idx| active.get(idx).copied().unwrap_or(false))
            .filter_map(|&idx| rules.get(idx).map(AsRef::as_ref))
            .collect();

        let mut has_rules = false;

        let enter: Vec<Vec<&'s dyn NativeRule>> = self
            .enter
            .iter()
            .map(|indices| {
                let mut resolved: Vec<&'s dyn NativeRule> = indices
                    .iter()
                    .filter(|&&idx| active.get(idx).copied().unwrap_or(false))
                    .filter_map(|&idx| rules.get(idx).map(AsRef::as_ref))
                    .collect();
                // Merge wildcard rules into each slot.
                resolved.extend_from_slice(&enter_all_resolved);
                if !resolved.is_empty() {
                    has_rules = true;
                }
                resolved
            })
            .collect();

        let leave: Vec<Vec<&'s dyn NativeRule>> = self
            .leave
            .iter()
            .map(|indices| {
                let mut resolved: Vec<&'s dyn NativeRule> = indices
                    .iter()
                    .filter(|&&idx| active.get(idx).copied().unwrap_or(false))
                    .filter_map(|&idx| rules.get(idx).map(AsRef::as_ref))
                    .collect();
                resolved.extend_from_slice(&leave_all_resolved);
                if !resolved.is_empty() {
                    has_rules = true;
                }
                resolved
            })
            .collect();

        CompactTable {
            enter,
            leave,
            has_rules,
        }
    }
}

// ---------------------------------------------------------------------------
// Pre-built traversal (session-level, zero per-file allocation)
// ---------------------------------------------------------------------------

/// Traverse the AST using a pre-built [`DispatchTable`] from `LintSession`.
///
/// Unlike [`traverse_and_lint_with_semantic`], this borrows session-level
/// structures and avoids per-file allocation of rule vecs and dispatch tables.
pub(crate) fn traverse_with_prebuilt<'a>(
    program: &Program<'a>,
    rules: &[Box<dyn NativeRule>],
    table: &DispatchTable,
    run_once_indices: &[usize],
    source_text: &'a str,
    file_path: &'a Path,
    semantic: Option<&'a Semantic<'a>>,
) -> Vec<Diagnostic> {
    // Per-file active-rule bitmask: call `should_run_on_file` once per rule.
    let active: Vec<bool> = rules
        .iter()
        .map(|r| r.should_run_on_file(source_text, file_path))
        .collect();

    let mut all_diagnostics = Vec::new();

    // Build a compact per-file dispatch table that stores direct rule
    // references, eliminating per-node active-mask checks and bounds checks.
    let compact = table.compact(rules, &active);

    // Run traversal-based rules via single-pass visitor with compact table.
    if !compact.is_empty() {
        let ctx = match semantic {
            Some(sem) => NativeLintContext::with_semantic(source_text, file_path, sem),
            None => NativeLintContext::new(source_text, file_path),
        };
        let mut visitor = CompactDispatchVisitor {
            table: &compact,
            ctx,
        };
        visitor.visit_program(program);
        all_diagnostics.extend(visitor.ctx.into_diagnostics());
    }

    // Run run_once for only the rules that override it (shared context, single pass).
    if !run_once_indices.is_empty() {
        let mut run_once_ctx = match semantic {
            Some(sem) => NativeLintContext::with_semantic(source_text, file_path, sem),
            None => NativeLintContext::new(source_text, file_path),
        };
        for &idx in run_once_indices {
            if active.get(idx).copied().unwrap_or(false) {
                if let Some(rule) = rules.get(idx) {
                    rule.run_once(&mut run_once_ctx);
                }
            }
        }
        all_diagnostics.extend(run_once_ctx.into_diagnostics());
    }

    all_diagnostics
}

// ---------------------------------------------------------------------------
// Compact per-file dispatch table
// ---------------------------------------------------------------------------

/// Per-file dispatch table storing direct rule references.
///
/// Built from the session-level [`DispatchTable`] by filtering out inactive
/// rules (those whose [`should_run_on_file`](NativeRule::should_run_on_file)
/// returned `false`). This eliminates per-node active-mask lookups and bounds
/// checks from the hot loop.
struct CompactTable<'s> {
    /// Per-AstType rule refs for `enter_node`. Index = `AstType as usize`.
    enter: Vec<Vec<&'s dyn NativeRule>>,
    /// Per-AstType rule refs for `leave_node`.
    leave: Vec<Vec<&'s dyn NativeRule>>,
    /// Whether any rules exist (avoid visiting the tree when nothing is active).
    has_rules: bool,
}

impl CompactTable<'_> {
    /// Returns `true` if no rules are active for this file.
    const fn is_empty(&self) -> bool {
        !self.has_rules
    }
}

// ---------------------------------------------------------------------------
// Visitors
// ---------------------------------------------------------------------------

/// Visitor using a compact per-file dispatch table (used by [`traverse_with_prebuilt`]).
///
/// All filtering (active mask, rule lookup) is done once at table construction;
/// the per-node loop is a simple iterate-and-call with no checks.
struct CompactDispatchVisitor<'a, 'file> {
    /// Compact per-file dispatch table with direct rule references.
    table: &'file CompactTable<'file>,
    /// Shared lint context — all rules push diagnostics into this.
    ctx: NativeLintContext<'a>,
}

impl<'a> Visit<'a> for CompactDispatchVisitor<'a, '_> {
    #[inline]
    fn enter_node(&mut self, kind: AstKind<'a>) {
        let ty = ast_type_index(kind.ty());
        // SAFETY of indexing: `ty` is always < AST_TYPE_MAX+1 since `AstType`
        // is `#[repr(u8)]` and the table was built with that size. We use
        // `.get()` for bounds safety but expect the compiler to elide the
        // check in release builds.
        if let Some(rules) = self.table.enter.get(ty) {
            for rule in rules {
                rule.run(&kind, &mut self.ctx);
            }
        }
    }

    #[inline]
    fn leave_node(&mut self, kind: AstKind<'a>) {
        let ty = ast_type_index(kind.ty());
        if let Some(rules) = self.table.leave.get(ty) {
            for rule in rules {
                rule.leave(&kind, &mut self.ctx);
            }
        }
    }
}

/// Visitor that dispatches AST nodes to interested rules only.
///
/// Uses a [`DispatchTable`] to route each node to the subset of rules that
/// declared interest, plus wildcard rules that receive everything.
/// Used by [`traverse_and_lint_with_semantic`] (backwards compat for tests).
struct RuleDispatchVisitor<'a, 'rules> {
    /// Active rules (indexed by dispatch table entries).
    rules: &'rules [&'rules dyn NativeRule],
    /// Interest-based dispatch table.
    table: DispatchTable,
    /// Shared lint context — all rules push diagnostics into this.
    ctx: NativeLintContext<'a>,
}

impl<'a> Visit<'a> for RuleDispatchVisitor<'a, '_> {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        let ty = ast_type_index(kind.ty());

        if let Some(targeted) = self.table.enter.get(ty) {
            for &idx in targeted {
                if let Some(rule) = self.rules.get(idx) {
                    rule.run(&kind, &mut self.ctx);
                }
            }
        }

        for &idx in &self.table.enter_all {
            if let Some(rule) = self.rules.get(idx) {
                rule.run(&kind, &mut self.ctx);
            }
        }
    }

    fn leave_node(&mut self, kind: AstKind<'a>) {
        let ty = ast_type_index(kind.ty());

        if let Some(targeted) = self.table.leave.get(ty) {
            for &idx in targeted {
                if let Some(rule) = self.rules.get(idx) {
                    rule.leave(&kind, &mut self.ctx);
                }
            }
        }

        for &idx in &self.table.leave_all {
            if let Some(rule) = self.rules.get(idx) {
                rule.leave(&kind, &mut self.ctx);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    use oxc_allocator::Allocator;
    use oxc_ast::ast_kind::AstType;
    use starlint_plugin_sdk::diagnostic::Span;
    use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

    use crate::parser::parse_file;

    /// A test rule that flags all `DebuggerStatement` nodes.
    #[derive(Debug)]
    struct NoDebuggerRule;

    impl NativeRule for NoDebuggerRule {
        fn meta(&self) -> RuleMeta {
            RuleMeta {
                name: "no-debugger".to_owned(),
                description: "Disallow debugger statements".to_owned(),
                category: Category::Correctness,
                default_severity: starlint_plugin_sdk::diagnostic::Severity::Error,
                fix_kind: FixKind::SafeFix,
            }
        }

        fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
            if let AstKind::DebuggerStatement(stmt) = kind {
                ctx.report_error(
                    "no-debugger",
                    "Unexpected `debugger` statement",
                    Span::new(stmt.span.start, stmt.span.end),
                );
            }
        }
    }

    /// Same as `NoDebuggerRule` but declares interest via `run_on_kinds`.
    #[derive(Debug)]
    struct TargetedNoDebuggerRule;

    impl NativeRule for TargetedNoDebuggerRule {
        fn meta(&self) -> RuleMeta {
            RuleMeta {
                name: "targeted-no-debugger".to_owned(),
                description: "Disallow debugger statements (targeted)".to_owned(),
                category: Category::Correctness,
                default_severity: starlint_plugin_sdk::diagnostic::Severity::Error,
                fix_kind: FixKind::SafeFix,
            }
        }

        fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
            if let AstKind::DebuggerStatement(stmt) = kind {
                ctx.report_error(
                    "targeted-no-debugger",
                    "Unexpected `debugger` statement",
                    Span::new(stmt.span.start, stmt.span.end),
                );
            }
        }

        fn run_on_kinds(&self) -> Option<&'static [AstType]> {
            Some(&[AstType::DebuggerStatement])
        }
    }

    #[test]
    fn test_traverse_finds_debugger() {
        let allocator = Allocator::default();
        let source = "debugger;\nconst x = 1;";
        let result = parse_file(&allocator, source, Path::new("test.js"));
        assert!(result.is_ok(), "parse should succeed");
        if let Ok(parsed) = result {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDebuggerRule)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 1, "should find one debugger statement");
        }
    }

    #[test]
    fn test_traverse_clean_file() {
        let allocator = Allocator::default();
        let source = "const x = 1;";
        let result = parse_file(&allocator, source, Path::new("test.js"));
        if let Ok(parsed) = result {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDebuggerRule)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 0, "clean file should have no diagnostics");
        }
    }

    #[test]
    fn test_targeted_rule_finds_debugger() {
        let allocator = Allocator::default();
        let source = "debugger;\nconst x = 1;";
        let result = parse_file(&allocator, source, Path::new("test.js"));
        assert!(result.is_ok(), "parse should succeed");
        if let Ok(parsed) = result {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(TargetedNoDebuggerRule)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 1, "targeted rule should find debugger");
        }
    }

    #[test]
    fn test_targeted_rule_clean_file() {
        let allocator = Allocator::default();
        let source = "const x = 1;";
        let result = parse_file(&allocator, source, Path::new("test.js"));
        if let Ok(parsed) = result {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(TargetedNoDebuggerRule)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 0, "targeted rule on clean file = no diags");
        }
    }

    #[test]
    fn test_mixed_wildcard_and_targeted_rules() {
        let allocator = Allocator::default();
        let source = "debugger;";
        let result = parse_file(&allocator, source, Path::new("test.js"));
        assert!(result.is_ok(), "parse should succeed");
        if let Ok(parsed) = result {
            let rules: Vec<Box<dyn NativeRule>> =
                vec![Box::new(NoDebuggerRule), Box::new(TargetedNoDebuggerRule)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 2, "both wildcard and targeted should fire");
        }
    }
}
