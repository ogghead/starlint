# Starlint Codebase Redundancy & Organization Analysis

**Date:** 2026-03-15
**Scope:** Full workspace (21 crates, 807 .rs files, ~137K lines)

---

## Executive Summary

The codebase is well-architected overall — the crate dependency graph is clean, the `declare_plugin!` macro eliminates boilerplate effectively, and the SDK/framework layering is intentional. However, analysis reveals **19 actionable redundancies** and **9 organizational improvements** across the workspace, concentrated in four areas: duplicated JSX attribute helpers (21 copies across react/nextjs), duplicated utility functions within and across plugin crates, unused/underutilized builder APIs, and a dual `Span` type situation.

---

## Category 1: Duplicate Utility Functions Across Plugin Crates

### R1. `is_pascal_case()` duplicated 4 times (HIGH)

**Identical or near-identical implementations in:**
- `crates/starlint_plugin_vue/src/rules/vue/component_definition_name_casing.rs:19`
- `crates/starlint_plugin_storybook/src/rules/storybook/prefer_pascal_case.rs:36`
- `crates/starlint_plugin_react/src/rules/react/jsx_pascal_case.rs:22`
- `examples/plugins/starlint-plugin-vue/src/lib.rs:485` (WASM example)

All check "starts with uppercase, no hyphens" with minor variations.

**Recommendation:** Extract to a `case_utils` module in `starlint_rule_framework` (or a new `starlint_text_utils` module re-exported from the framework). Functions: `is_pascal_case()`, `is_kebab_case()`, `is_camel_case()`, `is_snake_case()`.

### R2. `to_pascal_case()` duplicated 2 times (MEDIUM)

**Implementations in:**
- `crates/starlint_plugin_storybook/src/rules/storybook/prefer_pascal_case.rs:19`
- `crates/starlint_plugin_vue/src/rules/vue/component_definition_name_casing.rs:25`

Both split on `[-_ ]` and capitalize each segment. Slightly different implementations (one uses `to_uppercase()` char-by-char, other uses `format!`).

**Recommendation:** Consolidate alongside `is_pascal_case()` above.

### R3. `is_kebab_case()` duplicated 2 times (MEDIUM)

**Implementations in:**
- `crates/starlint_plugin_core/src/rules/filename_case.rs:25`
- `crates/starlint_plugin_vue/src/rules/vue/component_definition_name_casing.rs:43`

**Recommendation:** Same — consolidate into shared case utilities.

### R4. `is_camel_case()` + `to_camel_case()` (LOW)

**Implementation in:**
- `crates/starlint_plugin_vue/src/rules/vue/custom_event_name_casing.rs:21` (`to_camel_case`)
- `crates/starlint_plugin_vue/src/rules/vue/custom_event_name_casing.rs:40` (`is_camel_case`)

Currently only in one crate, but belongs with the other case utilities for consistency.

### R5. `is_inside_test_via_ancestors()` duplicated 3 times (HIGH)

**Identical implementations in:**
- `crates/starlint_plugin_testing/src/rules/jest/no_test_return_statement.rs:73`
- `crates/starlint_plugin_testing/src/rules/jest/no_conditional_in_test.rs:75`
- `crates/starlint_plugin_testing/src/rules/jest/no_standalone_expect.rs:80`

All three walk the parent chain looking for `test` or `it` call expressions. The only difference: `no_standalone_expect` uses a `TEST_CALLBACK_NAMES` constant (which includes `describe`, `beforeEach`, etc.) while the other two hardcode `"test"` and `"it"`.

**Recommendation:** Extract to a shared `jest_utils` module within `starlint_plugin_testing`. Parameterize the accepted function names.

### R6. `is_expect_chain()` / `is_expect_call_or_chain()` duplicated 5 times (HIGH)

**Implementations in:**
- `crates/starlint_plugin_testing/src/rules/jest/prefer_called_with.rs:94`
- `crates/starlint_plugin_testing/src/rules/jest/prefer_to_be.rs:130`
- `crates/starlint_plugin_testing/src/rules/jest/prefer_strict_equal.rs:89`
- `crates/starlint_plugin_testing/src/rules/jest/no_restricted_matchers.rs:95`
- `crates/starlint_plugin_testing/src/rules/jest/require_to_throw_message.rs:84`

All walk the call chain to determine if a call is part of an `expect()` chain.

**Recommendation:** Extract to the same `jest_utils` module.

### R7. `get_string_value()` duplicated 2 times (MEDIUM)

**Identical implementations in:**
- `crates/starlint_plugin_nextjs/src/rules/nextjs/google_font_preconnect.rs:22`
- `crates/starlint_plugin_nextjs/src/rules/nextjs/no_before_interactive_script_outside_document.rs:22`

Both extract a `StringLiteral` value from an optional `NodeId`. Exact same code.

**Recommendation:** Extract to a shared module within `starlint_plugin_nextjs` (e.g., `nextjs_utils`).

### R8. `is_promise_call()` duplicated 2 times (LOW)

**Implementations in:**
- `crates/starlint_plugin_testing/src/rules/jest/prefer_mock_return_shorthand.rs:144`
- `crates/starlint_plugin_testing/src/rules/jest/prefer_mock_promise_shorthand.rs:170`

Different signatures (one returns `bool`, other returns `Option<String>`) but overlapping logic.

**Recommendation:** Unify in `jest_utils` with the more general signature.

---

## Category 2: Unused or Underutilized Abstractions

### R9. `DiagnosticBuilder` is never used in any rule (HIGH)

**Location:** `crates/starlint_rule_framework/src/diagnostic_builder.rs` (196 lines)

The `DiagnosticBuilder` API exists with full test coverage, but **zero** of the 718 rules across 9 plugins use it. All 782 diagnostic emissions use raw `Diagnostic { ... }` struct literals.

**Recommendation:** Either:
1. **Adopt it** — migrate rules to use `DiagnosticBuilder` for consistency and to centralize default severity handling, OR
2. **Remove it** — delete dead code to reduce maintenance burden

Option 1 is preferable: the builder pattern would reduce boilerplate (especially the repeated `labels: vec![]` and `fix: None` fields) and make the API more ergonomic. A migration could be incremental — new rules use the builder, existing rules migrated per-plugin.

### R12. `has_attribute()` duplicated 13 times across JSX rules (HIGH)

**Identical or near-identical implementations in:**
- `crates/starlint_plugin_react/src/rules/jsx_a11y/alt_text.rs:24`
- `crates/starlint_plugin_react/src/rules/jsx_a11y/heading_has_content.rs:25`
- `crates/starlint_plugin_react/src/rules/jsx_a11y/anchor_is_valid.rs:22`
- `crates/starlint_plugin_react/src/rules/jsx_a11y/anchor_has_content.rs:22`
- `crates/starlint_plugin_react/src/rules/jsx_a11y/no_noninteractive_tabindex.rs:67`
- `crates/starlint_plugin_react/src/rules/jsx_a11y/aria_activedescendant_has_tabindex.rs:23`
- `crates/starlint_plugin_react/src/rules/jsx_a11y/label_has_associated_control.rs:20`
- `crates/starlint_plugin_react/src/rules/jsx_a11y/no_aria_hidden_on_focusable.rs:23`
- `crates/starlint_plugin_react/src/rules/jsx_a11y/role_has_required_aria_props.rs:35`
- `crates/starlint_plugin_react/src/rules/jsx_a11y/click_events_have_key_events.rs:20`
- `crates/starlint_plugin_react/src/rules/jsx_a11y/mouse_events_have_key_events.rs:20`
- `crates/starlint_plugin_react/src/rules/jsx_a11y/no_static_element_interactions.rs:63`
- `crates/starlint_plugin_react/src/rules/jsx_a11y/media_has_caption.rs:23`

All check whether a JSX element has a given attribute by name, walking the attributes `NodeId` list.

### R13. `get_attr_string_value()` duplicated 8 times across JSX rules (HIGH)

**Implementations in:**
- `crates/starlint_plugin_react/src/rules/jsx_a11y/alt_text.rs:35`
- `crates/starlint_plugin_react/src/rules/jsx_a11y/anchor_is_valid.rs:33`
- `crates/starlint_plugin_react/src/rules/jsx_a11y/anchor_ambiguous_text.rs:30`
- `crates/starlint_plugin_react/src/rules/jsx_a11y/aria_activedescendant_has_tabindex.rs:34`
- `crates/starlint_plugin_react/src/rules/jsx_a11y/role_supports_aria_props.rs:49`
- `crates/starlint_plugin_react/src/rules/react/jsx_no_target_blank.rs:21`
- `crates/starlint_plugin_nextjs/src/rules/nextjs/no_css_tags.rs:22` (+ 4 more nextjs rules)

**Recommendation for R12 + R13:** Extract to a new `jsx_utils` module in `starlint_rule_framework`:
```rust
pub fn has_jsx_attribute(attributes: &[NodeId], name: &str, ctx: &LintContext) -> bool
pub fn get_jsx_attr_string_value(attributes: &[NodeId], name: &str, ctx: &LintContext) -> Option<String>
```
This would deduplicate ~21 function definitions across 2 plugins and simplify ~25+ rule files.

### R14. `FixBuilder` exists but ~150 rules use raw `Fix` struct instead (MEDIUM)

`FixBuilder` (in `starlint_rule_framework::fix_builder`) is used by ~60 rules, but ~150 rules construct `Fix { kind, message, edits, is_snippet }` directly. The builder is more ergonomic and less error-prone (handles `is_snippet: false` default).

**Recommendation:** Migrate to `FixBuilder` incrementally alongside the `DiagnosticBuilder` migration (R9).

### R15. `static_key_name()` / `static_property_key_name()` duplicated 6 times in core plugin (HIGH)

**Implementations in:**
- `crates/starlint_plugin_core/src/rules/grouped_accessor_pairs.rs:207` — `static_property_key_name()`
- `crates/starlint_plugin_core/src/rules/no_dupe_keys.rs:83` — `static_property_key_name()`
- `crates/starlint_plugin_core/src/rules/accessor_pairs.rs:191` — `static_property_key_name()`
- `crates/starlint_plugin_core/src/rules/no_invalid_fetch_options.rs:112` — `static_key_name()`
- `crates/starlint_plugin_core/src/rules/no_dupe_class_members.rs:84` — `static_key_name()`
- `crates/starlint_plugin_core/src/rules/sort_keys.rs:19` — `key_name()`

All extract a key name from a property/method key `NodeId` (matching `IdentifierReference`, `BindingIdentifier`, `StringLiteral`, `NumericLiteral`). Minor variations in which node types each handles.

**Recommendation:** Extract to `starlint_rule_framework` or a shared `core_utils` module:
```rust
pub fn extract_static_key_name(key_id: NodeId, ctx: &LintContext) -> Option<String>
```

### R16. `is_literal()` duplicated 2 times in core plugin (LOW)

**Implementations in:**
- `crates/starlint_plugin_core/src/rules/prefer_class_fields.rs:119` — checks `NumericLiteral`, `StringLiteral`, `BooleanLiteral`, `NullLiteral`, `RegExpLiteral`, and `UnaryExpression` for negative numbers
- `crates/starlint_plugin_core/src/rules/yoda.rs:95` — checks `StringLiteral`, `NumericLiteral`, `BooleanLiteral`, `NullLiteral` only

**Recommendation:** Extract to shared utilities with two variants:
```rust
pub fn is_literal(node: &AstNode) -> bool
pub fn is_literal_or_unary(node: &AstNode) -> bool  // includes negative numbers, regex
```

### R10. `source_text_for_span()` overlaps with `Span::source_text()` (MEDIUM)

**Implementations:**
- `crates/starlint_rule_framework/src/fix_utils.rs:12` — `source_text_for_span(source, span)` free function
- `crates/starlint_ast/src/types.rs:72` — `Span::source_text(&self, source)` method

These do the same thing but the fix_utils version operates on SDK `Span` while the types version operates on AST `Span` (see R11 below). If the Span types are unified, one of these can be removed.

---

## Category 3: Dual Span Type (Architectural)

### R11. Two separate `Span` types with identical structure (HIGH)

**Definitions:**
- `starlint_plugin_sdk::diagnostic::Span` — `{ start: u32, end: u32 }`
- `starlint_ast::types::Span` — `{ start: u32, end: u32 }` (with additional methods: `len()`, `is_empty()`, `source_text()`, `EMPTY` constant)

The SDK Span is deliberately kept separate to avoid coupling the plugin API to the AST crate. However, this creates friction in `starlint_rule_framework` which uses **both** types:
- `LintContext` methods use SDK `Span` for diagnostics
- `resolve_symbol_id()` and `is_reference_resolved_at()` take AST `Span` (lines 187, 202)
- No `From`/`Into` conversion exists between them

**Recommendation:** Add bidirectional `From` impls between the two `Span` types in `starlint_rule_framework` (or in `starlint_ast` with SDK as a dependency, which is already the case). Alternatively, consider having `starlint_ast` depend on `starlint_plugin_sdk` and re-use the SDK Span directly.

---

## Category 4: Organizational Issues

### O1. `starlint_plugin_core` has 327 flat files with no subdirectory grouping (MEDIUM)

All 9 other plugins organize rules into subdirectories by domain:
- `starlint_plugin_react`: `react/`, `jsx_a11y/`, `react_perf/`
- `starlint_plugin_modules`: `import/`, `node/`, `promise/`
- `starlint_plugin_testing`: `jest/`, `vitest/`
- `starlint_plugin_typescript`: `typescript/`

But `starlint_plugin_core` puts all 327 rule files flat in `rules/`. This creates a massive `mod.rs` with 327 `pub mod` declarations and makes navigation difficult.

**Recommendation:** Group core rules into subdirectories by category. Potential groupings:
- `correctness/` — no-debugger, no-const-assign, constructor-super, etc.
- `style/` — arrow-body-style, curly, capitalized-comments, etc.
- `best_practices/` — eqeqeq, no-var, prefer-const, etc.
- `restriction/` — no-console, no-alert, etc.
- `suggestion/` — prefer-template, prefer-spread, etc.

### O2. `starlint_benches` has a 3,802-line `lib.rs` (LOW)

**Location:** `crates/starlint_benches/src/lib.rs` — 3,802 lines

This is a single file containing all benchmark definitions. While not a runtime concern, it's unwieldy for maintenance.

**Recommendation:** Split into per-plugin or per-category benchmark modules.

### O3. `starlint_ast/src/node.rs` at 4,258 lines (LOW)

The largest file in the codebase. Contains all `AstNode` enum variants and their associated types. Given that this is code-generated / highly structured data, the size is acceptable, but could benefit from being split by syntax category (expressions, statements, declarations, patterns) if it continues to grow.

### O4. Plugin crate structure is consistent except for `starlint_plugin_testing` having extra code in `lib.rs` (LOW)

All plugin `lib.rs` files are minimal — just `pub mod rules;` followed by `declare_plugin!`. But `starlint_plugin_testing/src/lib.rs` also defines `is_test_file()` (lines 8-27). This is a crate-level utility that rules use to check file naming patterns.

**Recommendation:** No action needed — this is a reasonable pattern for crate-scoped utilities. Document it as the recommended approach for other plugins that may need similar utilities.

### O5. `starlint_loader` rebuilds the native plugin registry twice per `load_plugins()` call (LOW)

**Location:** `crates/starlint_loader/src/lib.rs:120-134`

When `config.plugins.is_empty()`, `native_plugin_registry()` is called once on line 132. When plugins are explicitly configured, it's called on line 120 to build the `registry` HashMap, and potentially again on line 132 in the early-return path. The `native_plugin_registry()` function allocates a `Vec<NativePlugin>` each time.

**Recommendation:** Call `native_plugin_registry()` once and reuse. Minor performance improvement.

### O6. No shared test utilities module for plugin-common test patterns (MEDIUM)

Each plugin crate uses `starlint_rule_framework::lint_source()` (behind the `test-utils` feature) for rule testing. However, there's no shared test helper for common patterns like:
- Creating a mock file path
- Setting up a test config
- Asserting fix output

These are currently repeated or reimplemented in each plugin's test code.

**Recommendation:** Expand the `test-utils` feature in `starlint_rule_framework` with additional shared test helpers.

### O7. WASM example plugins duplicate native plugin logic (LOW)

**Locations:**
- `examples/plugins/starlint-plugin-storybook/src/lib.rs` — 358 lines
- `examples/plugins/starlint-plugin-vue/src/lib.rs` — 495 lines

These WASM examples contain their own `is_pascal_case()` and tree-walking helpers that duplicate logic from the native plugins. This is somewhat expected for examples, but the duplication could lead to inconsistencies.

**Recommendation:** Add comments noting that the examples are simplified versions and should not be kept in sync. Or consider using `include_str!` to link to shared documentation.

### O8. `starlint_core::diagnostic` has both `format_diagnostics()` (returns `String`) and `write_diagnostics()` (writes to `impl Write`) (LOW)

**Location:** `crates/starlint_core/src/diagnostic.rs:26-50`

Both functions exist for the same purpose. `write_diagnostics` is the streaming version used in production; `format_diagnostics` allocates a String and is used in tests.

**Recommendation:** No immediate action — both serve different use cases. Could consider removing `format_diagnostics()` if tests can be updated to use `write_diagnostics` with a `Vec<u8>` buffer.

### O9. File extension lists diverge between discovery and engine (MEDIUM — potential bug)

**Locations:**
- `crates/starlint_core/src/file_discovery.rs:10` — `DEFAULT_EXTENSIONS` = `["js", "jsx", "ts", "tsx", "mjs", "cjs", "mts", "cts"]` (8 extensions)
- `crates/starlint_core/src/engine.rs:201` — `is_supported_extension()` matches `["js", "mjs", "cjs", "jsx", "mjsx", "ts", "mts", "cts", "tsx", "mtsx"]` (10 extensions)

The engine supports `mjsx` and `mtsx` but file discovery does not. This means files with `.mjsx` or `.mtsx` extensions will never be discovered by directory walking, but would be accepted if passed directly. This is either a latent bug or an undocumented inconsistency.

**Recommendation:** Define a single `SUPPORTED_EXTENSIONS` constant in `file_discovery.rs` and reuse it in `engine.rs`. Decide whether `mjsx`/`mtsx` should be supported (they are non-standard but occasionally used).

---

## Priority Summary

| Priority | ID | Description | Impact |
|----------|-----|-------------|--------|
| HIGH | R9 | `DiagnosticBuilder` exists but unused (782 raw constructions) | Code consistency, reduced boilerplate |
| HIGH | R11 | Dual `Span` types without conversions | Developer friction |
| HIGH | R5 | `is_inside_test_via_ancestors()` x3 | Testing plugin maintenance |
| HIGH | R6 | `is_expect_chain()` x5 | Testing plugin maintenance |
| HIGH | R12 | `has_attribute()` x13 across JSX rules | Massive duplication |
| HIGH | R13 | `get_attr_string_value()` x8 across JSX rules | Massive duplication |
| HIGH | R15 | `static_key_name()` x6 in core plugin | Internal duplication |
| HIGH | R1 | `is_pascal_case()` x4 across plugins | Cross-plugin maintenance |
| MEDIUM | O1 | Core plugin 327 flat files, no subdirs | Developer navigation |
| MEDIUM | R2 | `to_pascal_case()` x2 | Consistency |
| MEDIUM | R3 | `is_kebab_case()` x2 | Consistency |
| MEDIUM | R7 | `get_string_value()` x2 in Next.js | Internal duplication |
| MEDIUM | R14 | `FixBuilder` exists but ~150 rules use raw `Fix` struct | Inconsistency |
| MEDIUM | R10 | `source_text_for_span` vs `Span::source_text` | API confusion |
| MEDIUM | O6 | No shared test utilities beyond `lint_source()` | Test boilerplate |
| MEDIUM | O9 | File extension lists diverge (8 vs 10) in starlint_core | Potential bug |
| LOW | R4 | Case utils only in Vue (preemptive) | Future duplication risk |
| LOW | R16 | `is_literal()` x2 in core plugin | Internal duplication |
| LOW | R8 | `is_promise_call()` x2 | Testing plugin cleanup |
| LOW | O2-O8 | Various organizational improvements | Maintainability |

---

## Suggested Implementation Order

1. **Quick wins (1-2 hours each):**
   - R5 + R6 + R8: Create `jest_utils` module in `starlint_plugin_testing`
   - R7: Create `nextjs_utils` module in `starlint_plugin_nextjs`
   - R11: Add `From` impls between the two `Span` types
   - O9: Unify file extension lists in `starlint_core`

2. **Medium effort (half-day each):**
   - R12 + R13: Create `jsx_utils` module in `starlint_rule_framework` (biggest impact — deduplicates 21 functions across 25+ files)
   - R15: Create `ast_utils` module with `extract_static_key_name()` (deduplicates 6 functions in core plugin)
   - R1 + R2 + R3 + R4: Create `case_utils` module in `starlint_rule_framework`
   - R10: Remove redundant `source_text_for_span()` once Spans are unified
   - O5: Single `native_plugin_registry()` call

3. **Larger initiatives (1-2 days each):**
   - R9 + R14: Decide on and implement `DiagnosticBuilder`/`FixBuilder` migration strategy
   - O1: Reorganize `starlint_plugin_core` rules into subdirectories
   - O6: Expand shared test utilities
