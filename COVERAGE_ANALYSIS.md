# Test Coverage Analysis

**Date:** 2025-03-15
**Overall Coverage:** 93.4% (7,343 / 111,778 lines missed)

## Executive Summary

The codebase has strong overall coverage at 93.4%, well above the 90% floor.
However, three infrastructure crates have critical gaps (LSP, CLI, WASM host),
and 45 lint rules fall below the 85% threshold. The gaps cluster around
integration boundaries—exactly the places where bugs are most costly.

---

## Priority 1: Infrastructure Crates with Critical Gaps

These are load-bearing crates where low coverage poses real risk.

### 1. `starlint_cli` — 6.6% coverage (410/439 lines missed)

**`src/lib.rs` is at 0%.** This is the main CLI orchestration entry point:
- `run()` — argument parsing → config → file discovery → linting → output
- `apply_fixes_to_files()` — multi-pass fix convergence loop
- `report_diagnostics()` — parallel counting + sequential formatting
- `run_init()`, `run_rules()`, `run_lsp()` — subcommand handlers
- `write_atomic()` — atomic file writes via temp files

**Recommended tests:**
- Integration tests that invoke `run()` with various CLI arg combinations
  against fixture directories (valid code, invalid code, config files)
- Unit tests for `apply_fixes_to_files()` convergence (0 passes, 1 pass,
  multi-pass, non-converging)
- Unit tests for `filter_fixable_diags()` and `report_diagnostics()`
- Snapshot tests for `run_rules()` output (plain and JSON formats)

### 2. `starlint_lsp` — 64.2% coverage (214/598 lines missed)

**`src/server.rs` is at 29%.** Only two utility helpers are tested. All LSP
protocol handlers are uncovered:
- `initialize`, `initialized`, `shutdown`
- `did_open`, `did_change`, `did_save`, `did_close`
- `code_action`
- `rebuild_session`, `lint_and_publish`

**Recommended tests:**
- Mock `tower-lsp` client tests (the tower-lsp crate supports this pattern)
- Test `lint_and_publish()` by constructing a `Backend` with a known config
  and verifying published diagnostics
- Test `code_action()` with pre-populated `cached_actions`
- Test `rebuild_session()` error paths (missing config, bad plugins)

### 3. `starlint_wasm_host` — 80.7% coverage (187/967 lines missed)

**`src/runtime.rs` is at 78%.** Conversion helpers are well-tested, but actual
WASM plugin lifecycle is not:
- `shared_engine()` — engine initialization/caching
- `load_plugin()`, `load_plugin_from_bytes()`, `finish_load()`
- `lint_with_plugin()` — full linting pipeline (AST serialization, fuel/memory limits)
- `create_store()`, `query_plugin_metadata()`, `validate_config()`

**Recommended tests:**
- Build a minimal test WASM component (a no-op plugin) and test the full
  load → configure → lint lifecycle
- Test fuel exhaustion and memory limit enforcement
- Test error paths: invalid WASM bytes, missing exports, bad config

---

## Priority 2: Core Infrastructure Gaps

### 4. `starlint_ast` — 87.2% coverage (434/3,393 lines missed)

**`src/node.rs` is at 73%.** The `span()` method is tested for 8 node types,
but ~70 `as_*()` type-narrowing methods have zero coverage. These are simple
match arms, so the risk is low, but they're exercised by every rule.

**Recommended tests:**
- A parameterized test that constructs each `Node` variant and verifies
  `as_X()` returns `Some` for the correct variant and `None` for others.
  This can be generated with a macro.

### 5. `starlint_loader` — 89.4% coverage (60/565 lines missed)

The WASM plugin loading paths (`#[cfg(feature = "wasm")]`) are conditionally
compiled and untested in default builds.

**Recommended tests:**
- Feature-gated integration tests that exercise WASM plugin discovery and loading
- Test `all_lint_rules()` and `all_rule_metas()` directly

### 6. `starlint_rule_framework/traversal.rs` — 89.1% coverage (60/550 lines missed)

`LintDispatchTable::filtered()` (per-file active-rule filtering) lacks direct tests.
Edge cases in dispatch interleaving and scope data handling are also uncovered.

**Recommended tests:**
- Test `filtered()` with rules that should/shouldn't apply to specific files
- Test traversal with `Some(scope_data)` vs `None`
- Test multiple rules targeting the same node type

### 7. `starlint_core/engine.rs` — 88.5% coverage (32/278 lines missed)

Builder methods, severity override application, file-pattern overrides, and
scope analysis triggering are not directly tested.

**Recommended tests:**
- Test `with_severity_overrides()` + `with_override_set()` + `with_disabled_rules()`
  chaining
- Test `lint_single_file()` with rules that produce diagnostics (requires a test
  plugin or using real plugins from the workspace)
- Test scope analysis path (`needs_scope_analysis = true`)

---

## Priority 3: Lint Rules Below 85% Coverage (45 rules)

These rules have significant untested code paths. The PR policy requires 95%
on new/changed lines, so these are legacy gaps. Sorted by coverage:

| Coverage | Rule | Plugin |
|----------|------|--------|
| 72.7% | `jsx_a11y/no_noninteractive_tabindex` | react |
| 73.3% | `no_this_in_exported_function` | core |
| 73.5% | `explicit_module_boundary_types` | typescript |
| 74.4% | `react/jsx_key` | react |
| 74.8% | `vitest/prefer_describe_function_title` | testing |
| 75.6% | `import/export` | modules |
| 76.1% | `import/extensions` | modules |
| 76.7% | `max_classes_per_file` | core |
| 76.9% | `jsx_a11y/lang` | react |
| 77.0% | `jsdoc/check_tag_names` | jsdoc |
| 77.6% | `no_fallthrough` | core |
| 78.1% | `react/no_unknown_property` | react |
| 78.2% | `no_unsafe_finally` | core |
| 78.3% | `const_comparisons` | core |
| 78.5% | `max_depth` | core |
| 78.6% | `no_misleading_character_class` | core |
| 78.7% | `jsx_a11y/alt_text` | react |
| 79.4% | `no_unused_labels` | core |
| 79.7% | `jest/prefer_mock_promise_shorthand` | testing |
| 79.8% | `no_this_before_super` | core |
| 80.3% | `prefer_optional_chain` | typescript |
| 80.7% | `no_thenable` | core |
| 81.0% | `no_unsafe_optional_chaining` | core |
| 81.0% | `react/only_export_components` | react |
| 81.0% | `jsdoc/match_name` | jsdoc |
| 81.2% | `curly` | core |
| 81.7% | `no_use_before_define` | core |
| 81.8% | `error_message` | core |
| 82.0% | `import/first` | modules |
| 82.2% | `react_perf/jsx_no_jsx_as_prop` | react |
| 82.3% | `no_promise_executor_return` | core |
| 82.4% | `no_invalid_void_type` | typescript |
| 82.6% | `import/no_default_export` | modules |
| 82.6% | `only_used_in_recursion` | core |
| 82.6% | `constructor_super` | core |
| 82.7% | `no_negated_condition` | core |
| 82.9% | `react/no_redundant_should_component_update` | react |
| 83.1% | `no_setter_return` | core |
| 83.4% | `max_statements` | core |
| 84.2% | `react/jsx_filename_extension` | react |
| 84.3% | `jest/no_done_callback` | testing |
| 84.4% | `react/no_array_index_key` | react |
| 84.6% | `no_useless_length_check` | core |
| 84.6% | `jest/valid_describe_callback` | testing |
| 84.8% | `jest/valid_title` | testing |

**By plugin breakdown:**
- core: 20 rules below 85%
- react: 9 rules below 85%
- testing: 5 rules below 85%
- modules: 4 rules below 85%
- typescript: 3 rules below 85%
- jsdoc: 2 rules below 85%

**Recommended approach:** For each rule, examine the untested branches — they
typically correspond to edge cases in JS/TS syntax (e.g., decorators, computed
properties, optional chaining, TypeScript generics). Add targeted test fixtures
exercising those paths.

---

## Priority 4: Parser Coverage Gaps

The parser is at 94.9% overall, which is good, but specific areas deserve attention:

| Coverage | File |
|----------|------|
| 90.6% | `token.rs` |
| 91.5% | `parser/typescript.rs` |
| 94.3% | `parser/modules.rs` |
| 95.1% | `parser/mod.rs` |
| 96.0% | `lexer.rs` |

The TypeScript parser at 91.5% likely has untested paths for uncommon TS syntax
(mapped types, conditional types, template literal types, etc.). These are
worth covering since parser bugs affect all downstream rules.

---

## Recommended Action Plan

1. **Highest ROI:** Add CLI integration tests (`starlint_cli`) — 0% → ~80%
   would cover the main user-facing entry point and catch regressions in
   argument handling, config loading, and output formatting.

2. **Highest risk:** Add LSP server tests (`starlint_lsp`) — a bug here
   affects every IDE user in real-time.

3. **Systematic rule improvement:** Batch the 45 rules below 85% into
   workstreams by plugin. Each rule typically needs 3-5 additional test
   fixtures to cover edge cases.

4. **WASM integration test:** Build a minimal test WASM component and
   exercise the full lifecycle. This is the only way to test `runtime.rs`.

5. **AST `as_*()` methods:** Generate tests via macro — mechanical but
   ensures the pattern matching is correct for all 70+ variants.
