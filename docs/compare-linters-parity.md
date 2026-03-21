# Starlint vs Oxlint vs ESLint — Feature Parity Analysis

**Date:** 2026-03-21
**Methodology:** Automated codebase inventory + web research of current documentation

---

## Executive Summary

Starlint ships **703 rules** across 9 plugins with **62 autofixable** (~8.8%), plus a
first-class WASM plugin system that neither competitor offers. Oxlint has **699 rules**
across 15 namespaces with **246 autofixable** (~35%), plus type-aware linting and
cross-file analysis. ESLint has ~240 frozen core rules but dominates in ecosystem
breadth (4,000+ community plugins).

Starlint's rule coverage is competitive. The critical gaps are **infrastructure features**
— inline disable comments, type-aware linting, autofix coverage, and shareable configs —
that block production adoption regardless of rule count.

---

## Rule Count Comparison

### Starlint (703 rules, 62 with autofix)

| Plugin | Rules | Autofix | Scope |
|--------|------:|--------:|-------|
| `starlint_plugin_core` | 324 | 23 | General JS/TS: best practices, style, correctness (includes 43 unicorn-equivalent rules) |
| `starlint_plugin_typescript` | 95 | 3 | TypeScript-specific rules |
| `starlint_plugin_react` | 87 | 23 | React + JSX a11y + React Perf |
| `starlint_plugin_testing` | 71 | 2 | Jest + Vitest |
| `starlint_plugin_modules` | 55 | 11 | Import + Node + Promise rules |
| `starlint_plugin_nextjs` | 21 | 0 | Next.js framework |
| `starlint_plugin_jsdoc` | 18 | 0 | JSDoc comments |
| `starlint_plugin_vue` | 17 | 0 | Vue framework |
| `starlint_plugin_storybook` | 15 | 0 | Storybook |
| **Total** | **703** | **62** | |

### Oxlint (699 rules, 246 with autofix)

| Plugin | Approx Rules | ESLint Equivalent |
|--------|-------------:|-------------------|
| eslint (core) | ~130 | ESLint core rules |
| typescript | ~90 | typescript-eslint |
| unicorn | ~80 | eslint-plugin-unicorn |
| react | ~70 | eslint-plugin-react + hooks + refresh |
| jest | ~50 | eslint-plugin-jest |
| jsx-a11y | ~30 | eslint-plugin-jsx-a11y |
| import | ~30 | eslint-plugin-import |
| oxc | ~25 | Unique (Clippy-inspired, no ESLint equivalent) |
| vitest | ~15 | @vitest/eslint-plugin |
| nextjs | ~15 | @next/eslint-plugin-next |
| vue | ~15 | eslint-plugin-vue (script blocks only) |
| promise | ~15 | eslint-plugin-promise |
| jsdoc | ~12 | eslint-plugin-jsdoc |
| node | ~5 | eslint-plugin-n |
| react-perf | ~4 | eslint-plugin-react-perf |
| **Total** | **~699** | |

Default-enabled: 108 rules (correctness category). Default plugins: eslint, typescript, unicorn, oxc.

### ESLint (~240 core rules)

- ~240 active core rules (frozen — few new additions)
- Major plugin ecosystem provides 1,000+ additional rules
- Key plugins: typescript-eslint (~130), react (~100), vue (~160+), jsx-a11y (~36), import (~34), jest (~50+), jsdoc (~58), next (~21), promise (~17), node (~30+)

### Per-Category Comparison

| Category | Starlint | Oxlint | Notes |
|----------|---------|--------|-------|
| Core JS/TS | 324 | ~130 | Starlint core includes unicorn-equivalent rules |
| TypeScript | 95 | ~90 | Comparable count; oxlint has 59 type-aware rules |
| React + a11y | 87 | ~100 | Oxlint separates react (~70) + jsx-a11y (~30) |
| Testing (Jest/Vitest) | 71 | ~65 | Comparable |
| Modules (Import/Node/Promise) | 55 | ~50 | Comparable; oxlint has cross-file `no-cycle` |
| Next.js | 21 | ~15 | Starlint leads |
| JSDoc | 18 | ~12 | Starlint leads |
| Vue | 17 | ~15 | Comparable |
| Storybook | 15 | 0 | **Starlint unique** |
| Unicorn (standalone) | 0 (43 in core) | ~80 | Starlint has 43 integrated; gap of ~37 rules |
| Oxc-specific | 0 | ~25 | **Oxlint unique** |

---

## Feature Comparison Matrix

| Feature | ESLint v10 | Oxlint v1.x | Starlint | Gap |
|---------|-----------|-------------|----------|-----|
| **Rule count** | ~240 core + ecosystem | ~699 built-in | 703 built-in | None |
| **Autofix coverage** | Varies by plugin | 246 rules (35%) | 62 rules (8.8%) | **Critical** |
| **Inline disable comments** | `eslint-disable` family | `oxlint-disable` + ESLint compat | `starlint-disable` family | None |
| **Type-aware linting** | Full (typescript-eslint) | 59/61 rules (via tsgo) | Not implemented | **Critical** |
| **Multi-file analysis** | Via plugins (import/no-cycle) | Built-in module graph | Not implemented | **High** |
| **Result caching** | `--cache` / `--cache-location` | Not needed (fast enough) | `--cache` | None |
| **Shareable configs** | `extends` in flat config | `extends` in config | Not implemented | **High** |
| **SARIF output** | Community formatter | Not implemented | Built-in | **Starlint leads** |
| **GitHub/GitLab output** | Via formatters | `--format github/gitlab` | Built-in | None |
| **JUnit output** | Community formatter | `--format junit` | Built-in | None |
| **Stylish output** | Default formatter | Built-in | Built-in | None |
| **Fix dry run** | `--fix-dry-run` | Not implemented | `--fix-dry-run` | None |
| **JS/TS plugin API** | Core feature | Alpha (JS plugins) | WASM plugins only | Medium |
| **WASM plugins** | Not supported | Not supported | **Built-in (wasmtime)** | **Starlint leads** |
| **Processors** | Extract from non-JS files | `.vue`/`.svelte`/`.astro` scripts | Not implemented | Medium |
| **Code path analysis** | Built-in CFG traversal | Not implemented | Not implemented | Medium |
| **Bulk suppressions** | `eslint-suppressions.json` | Not implemented | Not implemented | Medium |
| **Config inspector** | Web UI (`--inspect-config`) | Not implemented | Not implemented | Low |
| **Config migration tool** | Config Migrator | `@oxlint/migrate` | Not implemented | Low |
| **MCP server** | `@eslint/mcp` | Not implemented | Not implemented | Low |
| **Multithreaded linting** | `--concurrency` (v9.34+) | Built-in (Rust) | Built-in (rayon) | None |
| **Autofix** | `--fix` (single tier) | 3 tiers (safe/suggestion/dangerous) | 3 tiers (safe/suggestion/dangerous) | None |
| **Multi-pass fix** | Up to 10 iterations | Single pass | Up to 10 iterations | **Starlint leads** |
| **LSP** | VS Code extension only | VS Code extension | Built-in `starlint lsp` | **Starlint leads** |
| **Scope analysis** | eslint-scope | Built-in | Built-in (two-pass) | None |
| **TypeScript parsing** | Via typescript-eslint | Built-in (oxc parser) | Built-in (hand-written) | None |
| **Config format** | `eslint.config.js` (JS) | `.oxlintrc.json` / `.ts` | `starlint.toml` (TOML) | None |
| **File-pattern overrides** | Flat config objects | Config overrides | `[[overrides]]` blocks | None |
| **Output formats** | 4 built-in | 8 built-in | 9 built-in | **Starlint leads** |
| **Plugin feature gates** | N/A | N/A | Compile-time plugin selection | **Starlint unique** |
| **Storybook plugin** | Community | None | 15 rules | **Starlint unique** |
| **Monorepo nested config** | Per-dir lookup (v10) | Nested config discovery | Single config | Medium |
| **TypeScript config file** | N/A (JS config) | `oxlint.config.ts` | N/A (TOML) | Low |
| **`--stdin` support** | Built-in | Built-in | Not documented | Low |
| **`--quiet` skip execution** | Skips warn-level entirely | Not implemented | Not implemented | Low |
| **Multi-language linting** | JSON, CSS, Markdown, HTML | Not implemented | Not implemented | Low |

---

## Critical Gaps (Must-Have for Adoption)

### 1. Autofix Coverage — 62/703 (8.8%) vs Oxlint's 246/699 (35%)

**Impact:** Autofix is a primary productivity driver. Users expect `--fix` to handle
common issues (unused imports, missing semicolons, prefer-const, etc.) automatically.

**Current state:** Only 62 rules have autofix support. Framework plugins (nextjs, jsdoc,
vue, storybook) have zero autofixes. Core and react have 23 each.

**Target:** Reach ~200 autofixable rules (~28%) to match oxlint's ballpark. Prioritize:
- High-frequency style rules (formatting, naming)
- Import organization rules (already 11 in modules plugin)
- TypeScript rules with obvious fixes (prefer-as-const, consistent-type-imports)

**Effort estimate:** Most autofix implementations are 20-50 lines using `FixBuilder`.
A focused sprint could add 50-80 fixes across core and typescript plugins.

### 2. Type-Aware Linting

**What's missing:** Rules that use TypeScript's type checker (`no-floating-promises`,
`no-unsafe-assignment`, `await-thenable`, `no-misused-promises`, etc.).

**Why it matters:** Type-aware rules catch bugs that syntactic analysis cannot. They're
the primary reason teams use typescript-eslint. Oxlint now has 59/61 of these rules via
`tsgo` (TypeScript 7's Go port), running ~10x faster than ESLint.

**Recommended approach:**
- Integrate with `tsgo` for type information (same approach as oxlint)
- Start with the top 10 most-used type-aware rules
- Gate behind `--tsconfig` / `--type-aware` flag (opt-in)
- Consider incremental rollout: correctness rules first, then strictness rules

### 3. Shareable / Extendable Configs

**What's missing:** `extends` field in `starlint.toml`, config presets, named configs.

**Why it matters:** Teams share lint configurations across repos. Companies publish
org-wide configs. Without `extends`, every repo must duplicate configuration.

**Recommended approach:**
- Add `extends = ["./base.toml", "starlint:recommended"]` to config schema
- Support local file paths and built-in presets first
- Consider npm/crate-based config packages later

---

## High-Priority Gaps

### 4. Multi-File / Cross-File Analysis

**What's missing:** Module graph construction, cross-file import resolution.

**Impact:** Rules like `import/no-cycle`, `import/no-unresolved`, and `no-barrel-file`
require resolving imports across the project. Oxlint built a project-wide module graph
and ran `import/no-cycle` over 126K files in 7 seconds at Airbnb.

**Recommended approach:**
- Build module graph in a pre-pass before per-file linting
- Share resolution results across rules via `LintContext`
- Start with `no-cycle`, `no-unresolved`, `no-self-import`

### 5. Unicorn Rule Coverage Gap

**Current:** 43 unicorn-equivalent rules integrated into `starlint_plugin_core`.
**Oxlint:** ~80 unicorn rules. **Gap:** ~37 rules.

Oxlint enables unicorn rules by default (part of their default plugin set). The
`eslint-plugin-unicorn` upstream has ~141 rules total.

**Recommended approach:**
- Audit which of the ~37 missing unicorn rules are high-value
- Add them to `starlint_plugin_core` (maintain current integration pattern)
- Prioritize rules that are in oxlint's default-enabled set

---

## Medium-Priority Gaps

### 6. Processors / Script Block Extraction

**What's missing:** Extract and lint JS/TS from non-JS files (Vue SFCs, Svelte, Astro).

Starlint has 17 Vue rules, but they can only lint standalone `.js`/`.ts` files.
Oxlint extracts `<script>` blocks from `.vue`, `.svelte`, and `.astro` files natively.

### 7. Code Path Analysis

**What's missing:** Control flow graph (CFG) for rules like `no-unreachable`,
`no-fallthrough`, `consistent-return`.

ESLint provides `onCodePathStart`/`onCodePathEnd`/`onCodePathSegmentStart` events.
Neither oxlint nor starlint has this yet.

### 8. ESLint Plugin Compatibility

**What's missing:** Ability to run existing ESLint JS plugins.

Oxlint launched alpha JS plugin support (March 2026) — runs ESLint plugins 4.8-16x
faster than ESLint itself. Starlint's WASM plugin system is more sandboxed and portable,
but the ESLint ecosystem has 4,000+ JS-based packages.

### 9. Monorepo Nested Config

**What's missing:** Per-directory config cascade for monorepos.

ESLint v10 starts config lookup from the linted file's directory. Oxlint has nested
config discovery. Starlint uses a single root `starlint.toml`.

### 10. Bulk Suppressions

**What's missing:** A suppressions file for gradual rule adoption.

ESLint v9.24+ supports `eslint-suppressions.json` — a file that records existing
violations so new rules can be enforced on new code without fixing all legacy issues.

---

## Low-Priority / Nice-to-Have

| Feature | Notes |
|---------|-------|
| Config migration tool | `eslint-to-starlint` converter for adoption |
| TypeScript config file | `starlint.config.ts` (oxlint supports this) |
| `--stdin` support | Lint from stdin for editor integrations |
| Config inspector | Web UI for debugging config |
| `--quiet` skip execution | Skip warn-level rule execution entirely (perf win) |
| Multi-language linting | JSON, CSS, Markdown, HTML (ESLint v10 supports this) |
| MCP server | AI tool integration for lint-aware code generation |
| Oxc-equivalent rules | ~25 unique Clippy-inspired rules (e.g., `const-comparisons`, `approx-constant`) |

---

## Starlint Unique Advantages

These are features where starlint leads both competitors:

| Advantage | Details |
|-----------|---------|
| **WASM plugin system** | wasmtime 42, Component Model, WIT interface. Sandboxed (10M instructions, 16MB memory per file). Neither ESLint nor oxlint supports WASM plugins. |
| **Output format breadth** | 9 built-in formats including SARIF (oxlint: 8, no SARIF; ESLint: 4 built-in) |
| **Multi-pass autofix** | Up to 10 convergence passes with overlap detection. ESLint also does 10 passes, but oxlint is single-pass. |
| **Built-in LSP binary** | Ships as `starlint lsp` subcommand, not a separate VS Code-only extension |
| **TOML config** | No JS execution required for config loading; simpler, faster, more predictable |
| **Plugin feature gates** | Compile custom distributions with only needed plugins via Cargo features |
| **Flat indexed AST** | `NodeId`-based, JSON-serializable, no lifetimes — enables WASM serialization |
| **Storybook plugin** | 15 rules; neither ESLint core nor oxlint covers Storybook |
| **Higher rule density** | 703 rules in 9 plugins vs oxlint's 699 in 15 namespaces |

---

## Recommended Roadmap

### Phase 1 — Adoption Unblockers (Immediate)
1. **Autofix expansion** — Add fixes to 50-80 high-frequency rules (target: ~140 total)
2. **Shareable configs** — `extends` field in `starlint.toml`

### Phase 2 — Competitive Parity (Near-term)
3. **Unicorn rule coverage** — Close the ~37-rule gap with oxlint
4. **Script block extraction** — `.vue`/`.svelte`/`.astro` support
5. **Monorepo nested config** — Per-directory config cascade

### Phase 3 — Advanced Analysis (Medium-term)
6. **Type-aware linting** — Integrate `tsgo`, start with top 10 rules
7. **Multi-file analysis** — Module graph for `no-cycle`, `no-unresolved`
8. **Code path analysis** — CFG for `no-unreachable`, `no-fallthrough`

### Phase 4 — Ecosystem Growth (Long-term)
9. **ESLint plugin compatibility** — Run JS plugins via subprocess or WASM bridge
10. **Config migration tool** — `eslint-to-starlint` converter
11. **Bulk suppressions** — Gradual rule adoption support
12. **Oxc-equivalent rules** — Clippy-inspired unique rules

---

## Autofix Coverage Detail

### Rules with Autofix by Plugin

**starlint_plugin_core (23/324):**
Rules with `FixBuilder`/`with_fix` — primarily style and import-related fixes.

**starlint_plugin_react (23/87):**
Highest autofix ratio (26%) — covers JSX best practices, hook dependencies.

**starlint_plugin_modules (11/55):**
Import organization, sorting, deduplication fixes.

**starlint_plugin_typescript (3/95):**
Minimal — significant expansion opportunity for type annotation fixes.

**starlint_plugin_testing (2/71):**
Minimal — opportunity for test assertion style fixes.

**Framework plugins (0 fixes across nextjs/jsdoc/vue/storybook):**
No autofix support. Lower priority given smaller rule counts.

### Comparison with Oxlint Autofix

| Metric | Starlint | Oxlint |
|--------|---------|--------|
| Total rules with autofix | 62 | 246 |
| Autofix percentage | 8.8% | 35.2% |
| Fix tiers | 3 (safe/suggestion/dangerous) | 3 (safe/suggestion/dangerous) |
| Multi-pass convergence | Up to 10 passes | Single pass |
| Overlapping edit handling | Skip overlaps, retry next pass | Not documented |

---

## Methodology

- **Starlint data:** Automated codebase scan of `declare_plugin!` macros and `FixBuilder`/`with_fix` usage across all 9 plugin crates.
- **Oxlint data:** Official documentation (oxc.rs), blog posts, GitHub issues, npm packages. Rule counts are approximate as oxlint does not publish a single canonical per-plugin breakdown.
- **ESLint data:** Official documentation (eslint.org), blog posts, typescript-eslint docs.

## Sources

- [Oxlint Rules Reference](https://oxc.rs/docs/guide/usage/linter/rules)
- [Oxlint Built-in Plugins](https://oxc.rs/docs/guide/usage/linter/plugins)
- [Oxlint Automatic Fixes](https://oxc.rs/docs/guide/usage/linter/automatic-fixes)
- [Oxlint v1.0 Stable](https://voidzero.dev/posts/announcing-oxlint-1-stable)
- [Oxlint JS Plugins Alpha](https://oxc.rs/blog/2026-03-11-oxlint-js-plugins-alpha)
- [Oxlint Type-Aware Alpha](https://oxc.rs/blog/2025-12-08-type-aware-alpha)
- [Oxlint Beta Post](https://oxc.rs/blog/2025-03-15-oxlint-beta)
- [Migrate from ESLint](https://oxc.rs/docs/guide/usage/linter/migrate-from-eslint)
- [ESLint CLI Reference](https://eslint.org/docs/latest/use/command-line-interface)
- [ESLint Formatters](https://eslint.org/docs/latest/use/formatters/)
- [ESLint Flat Config Extends](https://eslint.org/blog/2025/03/flat-config-extends-define-config-global-ignores/)
- [ESLint Multithread Linting](https://eslint.org/blog/2025/08/multithread-linting/)
- [ESLint Bulk Suppressions](https://eslint.org/blog/2025/04/introducing-bulk-suppressions/)
- [ESLint MCP Server](https://eslint.org/docs/latest/use/mcp)
- [ESLint Code Path Analysis](https://eslint.org/docs/latest/extend/code-path-analysis)
- [typescript-eslint Typed Linting](https://typescript-eslint.io/getting-started/typed-linting/)
- [eslint-plugin-oxlint (npm)](https://www.npmjs.com/package/eslint-plugin-oxlint)
