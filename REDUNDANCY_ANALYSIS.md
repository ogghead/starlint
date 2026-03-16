# Starlint Codebase Redundancy & Organization Analysis

**Date:** 2026-03-16
**Scope:** All 21 crates, 809 Rust files, 718 lint rules

---

## Executive Summary

The starlint workspace is **architecturally sound** with clean crate boundaries, a proper DAG dependency graph, no circular dependencies, and excellent workspace-level dependency consolidation. However, there are significant opportunities for improvement in three areas:

1. **Duplicate rules across plugins** (4 confirmed, 3 naming conflicts)
2. **Rule-level boilerplate** (732 rules with repeated diagnostic/metadata patterns)
3. **Two divergent Span types** causing conversion friction across crates

The recommendations below are ordered by impact and effort.

---

## Table of Contents

1. [Critical: Duplicate Rules Across Plugins](#1-critical-duplicate-rules-across-plugins)
2. [High: Dual Span Types](#2-high-dual-span-types)
3. [High: Rule Boilerplate Reduction](#3-high-rule-boilerplate-reduction)
4. [Medium: Source Text Extraction Pattern](#4-medium-source-text-extraction-pattern)
5. [Medium: Parser File Decomposition](#5-medium-parser-file-decomposition)
6. [Medium: AST Helper Utilities](#6-medium-ast-helper-utilities)
7. [Low: Plugin Organization Consistency](#7-low-plugin-organization-consistency)
8. [Low: Test Pattern Standardization](#8-low-test-pattern-standardization)
9. [Low: Config Validation](#9-low-config-validation)
10. [Informational: What's Already Good](#10-informational-whats-already-good)

---

## 1. Critical: Duplicate Rules Across Plugins

### Problem

4 rules have **duplicate implementations** in both `starlint_plugin_core` and `starlint_plugin_typescript` with **different detection strategies**:

| Rule | Core Implementation | TypeScript Implementation |
|------|-------------------|--------------------------|
| `prefer-includes` | Full AST matching (`run_on_types` + `run`) | Syntax-only source scanning (`run_once`) |
| `prefer-promise-reject-errors` | Full AST traversal | Syntax-only scan |
| `prefer-string-starts-ends-with` | Full AST traversal | Syntax-only scan |
| `require-await` | Full AST traversal | Syntax-only scan |

Additionally, 3 rules have **naming collisions** across plugins:

| Rule Name | Plugins | Issue |
|-----------|---------|-------|
| `no-anonymous-default-export` | core, modules (import) | Nearly identical logic, different prefixes |
| `no-process-exit` | core, modules (node) | Same check, different categories (Style vs Suggestion) |
| `no-namespace` | typescript, react, modules (import) | Three unrelated rules sharing a name |

### Files

- `crates/starlint_plugin_core/src/rules/prefer_includes.rs`
- `crates/starlint_plugin_typescript/src/rules/typescript/prefer_includes.rs`
- `crates/starlint_plugin_core/src/rules/no_anonymous_default_export.rs`
- `crates/starlint_plugin_modules/src/rules/import/no_anonymous_default_export.rs`
- `crates/starlint_plugin_core/src/rules/no_process_exit.rs`
- `crates/starlint_plugin_modules/src/rules/node/no_process_exit.rs`

### Impact

Users enabling both plugins get **different violations** for the same code depending on which plugin runs first. This undermines trust in lint results.

### Recommendation

1. **Establish an ownership model**: Framework-level rules live in core; domain-specific variants live in their plugin with the plugin prefix.
2. **Consolidate the 4 duplicate rules**: Keep the more thorough (AST-based) implementation in one location. If TypeScript needs type-aware behavior, extend the core rule with a configuration flag.
3. **Resolve naming conflicts**: Either remove the core duplicate or ensure distinct rule names with clear documentation about which to use.

---

## 2. High: Dual Span Types

### Problem

Two nearly identical `Span` types exist:

- **`starlint_ast::types::Span`** — has utility methods: `len()`, `is_empty()`, `source_text()`, constant `EMPTY`
- **`starlint_plugin_sdk::diagnostic::Span`** — data-only (start/end), no methods

Rules receive `starlint_ast::types::Span` from AST operations but must use `starlint_plugin_sdk::diagnostic::Span` for diagnostics and fixes. This forces aliasing and manual field copying:

```rust
// From crates/starlint_scope/src/scope_data.rs:7
use starlint_ast::types::Span;
use starlint_plugin_sdk::diagnostic::Span as DiagSpan;
```

### Files

- `crates/starlint_ast/src/types.rs` (lines 39-77)
- `crates/starlint_plugin_sdk/src/diagnostic.rs` (lines 10-25)
- `crates/starlint_scope/src/scope_data.rs` (line 7, aliasing)
- `crates/starlint_rule_framework/src/fix_utils.rs` (uses SDK Span)
- `crates/starlint_rule_framework/src/lint_rule.rs` (uses AST Span)

### Recommendation

**Option A (preferred)**: Add `impl From<starlint_ast::types::Span> for starlint_plugin_sdk::diagnostic::Span` to enable zero-friction conversion. This is the smallest change.

**Option B**: Unify into a single Span type in `starlint_plugin_sdk` (since SDK is lower in the dependency graph than AST), then have AST re-export it. Move utility methods to the unified type.

---

## 3. High: Rule Boilerplate Reduction

### Problem

All 732 rules repeat the same patterns for diagnostic construction and metadata:

**Diagnostic construction** (appears in every rule):
```rust
ctx.report(Diagnostic {
    rule_name: "rule-name".to_owned(),
    message: "message".to_owned(),
    span: Span::new(expr.span.start, expr.span.end),
    severity: Severity::Error,
    help: None,
    fix: None,
    labels: vec![],
});
```

**Metadata construction** (appears in every rule):
```rust
fn meta(&self) -> RuleMeta {
    RuleMeta {
        name: "rule-name".to_owned(),
        description: "...".to_owned(),
        category: Category::Correctness,
        default_severity: Severity::Error,
    }
}
```

### Underutilized Existing Infrastructure

- `DiagnosticBuilder` exists at `crates/starlint_rule_framework/src/diagnostic_builder.rs` but is used by **<2% of rules**
- `LintContext::report_error()` and `report_warning()` exist at `crates/starlint_rule_framework/src/lint_rule.rs:225-248` but are **rarely used**

### Recommendation

1. **Promote existing helpers**: Update rule templates and documentation to use `ctx.report_error()` / `ctx.report_warning()` / `DiagnosticBuilder` instead of manual `Diagnostic { }` construction. This is a documentation + convention change — zero code changes needed to the framework.

2. **Consider a derive macro** for metadata (longer term):
   ```rust
   #[derive(LintRule)]
   #[rule(name = "no-debugger", category = "Correctness", severity = "Error",
          description = "Disallow the use of debugger")]
   pub struct NoDebugger;
   ```

3. **Estimated impact**: Reduces per-rule boilerplate by ~10 lines (from ~20 to ~10 lines of framework interaction code).

---

## 4. Medium: Source Text Extraction Pattern

### Problem

~150+ rules manually extract source text from spans using a verbose 6-8 line pattern:

```rust
let start = usize::try_from(span.start).unwrap_or(0);
let end = usize::try_from(span.end).unwrap_or(0);
let source = ctx.source_text();
let text = source.get(start..end);
```

Meanwhile, `fix_utils::source_text_for_span()` exists at `crates/starlint_rule_framework/src/fix_utils.rs:8-23` but is primarily associated with fix generation and not widely known.

### Files

- `crates/starlint_rule_framework/src/fix_utils.rs` (existing helper)
- `crates/starlint_rule_framework/src/lint_rule.rs` (LintContext)

### Recommendation

Add convenience methods to `LintContext`:

```rust
impl LintContext<'_> {
    pub fn text_for_span(&self, span: Span) -> Option<&str> { ... }
    pub fn text_for_node(&self, id: NodeId) -> Option<&str> { ... }
}
```

**Effort**: Small (add 2 methods). **Impact**: Eliminates ~900 lines of duplicated span-to-text extraction across 150+ rules.

---

## 5. Medium: Parser File Decomposition

### Problem

Two parser files are large:

| File | Lines | Content |
|------|-------|---------|
| `crates/starlint_parser/src/parser/expressions.rs` | 1,549 | Pratt parsing + primary atoms + member/call + arrow functions + JSX expressions |
| `crates/starlint_parser/src/parser/statements.rs` | 1,360 | All statement types (declarations, control flow, loops, classes) |

### Recommendation

Split into logical submodules:

```
parser/expressions/
  mod.rs         (shared APIs, re-exports)
  pratt.rs       (precedence climbing, binary/logical)
  primary.rs     (atoms, literals, identifiers)
  postfix.rs     (call, member, update)
  jsx_expr.rs    (JSX-specific)

parser/statements/
  mod.rs
  declaration.rs
  control_flow.rs  (if/switch/try)
  loops.rs
  classes.rs
```

**Risk**: Medium — requires careful testing to ensure no regressions. No functional changes, purely organizational.

---

## 6. Medium: AST Helper Utilities

### Problem

Rules frequently define local helper functions for common AST checks that could be shared:

- `is_bitwise_binary()` — `crates/starlint_plugin_core/src/rules/no_bitwise.rs:68-78`
- `is_path_global()` — `crates/starlint_plugin_modules/src/rules/node/no_path_concat.rs:21-27`
- `contains_assignment()` — `crates/starlint_plugin_core/src/rules/no_return_assign.rs:56-63`

~87+ such helpers exist across `starlint_plugin_core` rules alone.

### Recommendation

Create `crates/starlint_rule_framework/src/ast_utils.rs` with common predicates:

```rust
pub fn is_identifier_named(node: Option<&AstNode>, name: &str) -> bool { ... }
pub fn is_member_expression_of(node: &AstNode, object: &str, property: &str) -> bool { ... }
pub const fn is_bitwise_operator(op: BinaryOperator) -> bool { ... }
pub fn extract_string_literal(node: &AstNode) -> Option<&str> { ... }
```

**Effort**: Medium (audit existing helpers, extract common ones). **Impact**: Reduces code duplication across 100+ rules and provides a discoverable API for rule authors.

---

## 7. Low: Plugin Organization Consistency

### Problem

Three different organizational patterns exist across the 9 plugins:

| Pattern | Plugins | Structure |
|---------|---------|-----------|
| Flat | core (327 rules) | `rules/*.rs` |
| Single subdir | typescript, nextjs, jsdoc, vue, storybook | `rules/<category>/*.rs` |
| Multi subdir | react, testing, modules | `rules/<cat1>/*.rs`, `rules/<cat2>/*.rs` |

### Recommendation

This is **not a blocker** — all patterns work. However, for consistency:

- **Document the convention**: Flat for single-concern plugins, subdirectories for multi-concern plugins
- **Consider splitting core**: With 327 rules, `starlint_plugin_core` could benefit from subdirectories (e.g., `rules/best_practices/`, `rules/style/`, `rules/correctness/`)

---

## 8. Low: Test Pattern Standardization

### Problem

Two testing patterns coexist:

- **Pattern A (Manual)**: Direct `lint_source()` call (some core plugin rules)
- **Pattern B (Macro)**: `lint_rule_test!()` macro that generates a `lint()` helper (most plugins)

Both work equally well. Total test count: 1,368 `#[test]` functions across all plugins.

### Recommendation

- Standardize on `lint_rule_test!()` macro for new rules
- Document the preferred pattern in CONTRIBUTING.md
- Optionally migrate existing manual tests (low priority, high effort for marginal gain)

---

## 9. Low: Config Validation

### Problem

`crates/starlint_config/src/lib.rs` deserializes configuration but does not validate that:
- Rule names in config actually exist in loaded plugins
- Plugin names in config correspond to available plugins

Errors surface at runtime during lint execution.

### Recommendation

Add a validation stage after deserialization:

```rust
impl Config {
    pub fn validate(&self, available_rules: &[&str]) -> Result<(), Vec<ConfigWarning>> { ... }
}
```

Call during `LintSession` initialization to provide early, actionable error messages.

---

## 10. Informational: What's Already Good

These areas were analyzed and found to be **well-designed with no actionable issues**:

| Area | Status | Notes |
|------|--------|-------|
| **Workspace dependency consolidation** | Excellent | 100% centralized, no version mismatches |
| **Feature flag design** | Optimal | Plugin-level gating in loader, minimal builds possible |
| **Crate dependency graph** | Clean DAG | No circular dependencies |
| **Public API discipline** | Good | Minimal re-exports, strategic `pub(crate)` usage |
| **Error handling patterns** | Consistent | All crates use miette + thiserror uniformly |
| **Dead code** | None detected | All modules and dependencies justified |
| **Plugin registration** | Excellent | `declare_plugin!` macro eliminates boilerplate |
| **Rule metadata consistency** | Very high | Identical `RuleMeta` structure across all 732 rules |
| **Build configuration** | Clean | No build.rs files, profiles well-defined |
| **Test utilities** | Correct | Feature-gated `test-utils` doesn't leak into production |
| **Scope analysis** | Well-contained | Clean two-pass builder, lazy evaluation |
| **Traversal dispatch** | Efficient | Single-pass with `interested_node_types()` filtering |

---

## Prioritized Action Plan

| # | Item | Impact | Effort | Risk |
|---|------|--------|--------|------|
| 1 | Resolve 4 duplicate rules (core vs typescript) | High | Low | Low |
| 2 | Add `From` impl between Span types | High | Low | Low |
| 3 | Promote `report_error()`/`report_warning()`/`DiagnosticBuilder` in docs | High | Low | None |
| 4 | Add `text_for_span()`/`text_for_node()` to `LintContext` | Medium | Low | Low |
| 5 | Create shared `ast_utils.rs` module | Medium | Medium | Low |
| 6 | Resolve 3 rule naming conflicts | Medium | Low | Low |
| 7 | Split parser expression/statement files | Medium | Medium | Medium |
| 8 | Add config validation stage | Low | Low | Low |
| 9 | Standardize test patterns in docs | Low | Low | None |
| 10 | Reorganize core plugin into subdirectories | Low | Medium | Low |
