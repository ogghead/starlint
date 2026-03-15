# Starlint vs Oxlint vs ESLint — Feature Parity Analysis

**Date:** 2026-03-15

## Executive Summary

Starlint (718 rules) has strong rule coverage — exceeding oxlint (~695 rules) and
rivaling ESLint's core (~300 built-in rules). However, several **infrastructure
features** present in one or both competitors are missing from starlint. The gaps
below are ranked by user impact.

---

## Feature Comparison Matrix

| Feature | ESLint v9 | Oxlint v1.x | Starlint | Gap Severity |
|---|---|---|---|---|
| **Rule count** | ~300 core + ecosystem | ~695 built-in | 718 built-in | None |
| **Inline disable comments** | `eslint-disable`, `eslint-disable-next-line`, `eslint-disable-next-line <rule>` | `oxlint-disable` + ESLint compat | Not implemented | **Critical** |
| **Type-aware linting** | Full (typescript-eslint) | 59/61 rules (via tsgo) | Not implemented | **Critical** |
| **Multi-file / cross-file analysis** | Via plugins (import/no-cycle) | Built-in module graph | Not implemented | **High** |
| **Result caching** | `--cache` / `--cache-location` | Not needed (fast enough) | Not implemented | **High** |
| **SARIF output** | Via community formatter | Not implemented | Not implemented | Medium |
| **Shareable/extendable configs** | `extends` in flat config | `extends` in config | Not implemented | **High** |
| **Config `extends`** | Flat config `extends` array | `.oxlintrc.json` extends | Not implemented | **High** |
| **Stylish output format** | Default formatter | Built-in | Not implemented (has Pretty) | Medium |
| **GitHub/GitLab output** | Via formatters | `--format github`, `--format gitlab` | Not implemented | Medium |
| **JUnit output** | Via community formatter | `--format junit` | Not implemented | Medium |
| **Checkstyle output** | Via community formatter | `--format checkstyle` | Not implemented | Low |
| **Unix output** | Via community formatter | `--format unix` | Not implemented (has Compact) | Low |
| **JS/TS plugin API** | Core feature | Alpha (JS plugins) | WASM plugins only | Medium |
| **Processors** | Extract JS from non-JS files | Not implemented | Not implemented | Medium |
| **Code path analysis** | Built-in (unreachable code, etc.) | Not implemented | Not implemented | Medium |
| **`--fix-dry-run`** | Built-in | Not implemented | Not implemented | Medium |
| **Unused disable directive reporting** | `reportUnusedDisableDirectives` | `--report-unused-disable-directives` | N/A (no directives yet) | Blocked |
| **Config migration tools** | Config Migrator, Inspector | `oxlint-migrate` from ESLint | Not implemented | Low |
| **Parallel linting** | Community (`eslint-parallel`) | Built-in (Rust) | Built-in (rayon) | None |
| **Autofix** | `--fix` | `--fix` | `--fix` (multi-pass) | None |
| **LSP** | VS Code extension | Oxc VS Code extension | Built-in LSP server | None |
| **Scope analysis** | eslint-scope | Built-in | Built-in (two-pass) | None |
| **TypeScript parsing** | Via typescript-eslint parser | Built-in (oxc parser) | Built-in (hand-written) | None |
| **JSX/TSX support** | Via plugins | Built-in | Built-in | None |
| **WASM plugins** | Not supported | Not supported | **Unique advantage** | N/A |
| **Custom parser** | Pluggable parsers | Fixed (oxc) | Fixed (hand-written) | None |
| **Config format** | `eslint.config.js` (JS) | `.oxlintrc.json` / `.ts` | `starlint.toml` (TOML) | None |
| **File-pattern overrides** | Flat config objects | Config overrides | `[[overrides]]` blocks | None |
| **Max warnings** | `--max-warnings` | `--max-warnings` | `--max-warnings` | None |
| **Timing/stats** | `--stats` | `--timing` | `--timing` | None |
| **Init command** | `--init` | `--init` | `init` subcommand | None |

---

## Critical Gaps (Must-Have for Adoption)

### 1. Inline Disable Comments

**What's missing:** `starlint-disable`, `starlint-disable-next-line`, `starlint-disable-next-line <rule>`, `starlint-enable`.

**Why it matters:** Every production codebase needs to suppress false positives or
intentional violations. Without this, users cannot adopt starlint as a primary
linter. This is the single biggest blocker.

**ESLint behavior:**
- `/* eslint-disable */` — disable all rules for rest of file
- `/* eslint-disable no-console */` — disable specific rule
- `// eslint-disable-next-line` — disable for next line only
- `/* eslint-enable */` — re-enable rules
- `noInlineConfig` setting to forbid inline comments
- `reportUnusedDisableDirectives` to flag stale suppressions

**Oxlint behavior:** Mirrors ESLint, also accepts `eslint-disable` for migration compat.

**Recommended implementation:**
- Parse comments during/after AST construction
- Build a directive map (line ranges → disabled rules)
- Filter diagnostics post-lint using the directive map
- Support `starlint-disable`, `starlint-disable-next-line`, `starlint-enable`
- Consider `eslint-disable` compat mode for migration

### 2. Type-Aware Linting

**What's missing:** Rules that use TypeScript's type checker (e.g., `no-floating-promises`,
`no-unsafe-assignment`, `await-thenable`, `no-misused-promises`).

**Why it matters:** Type-aware rules catch bugs that syntactic analysis cannot. They're
the primary reason teams use typescript-eslint. Without them, starlint cannot fully
replace ESLint for TypeScript projects.

**ESLint:** 61 type-aware rules via typescript-eslint (requires `tsconfig.json` and
a running type checker).

**Oxlint:** 59/61 type-aware rules, powered by `tsgo` (TypeScript 7's Go port).
Performance is ~10x faster than ESLint's type-aware linting.

**Recommended approach:**
- Integrate with `tsgo` or `swc` for type information
- Start with the top 10 most-used type-aware rules
- Gate behind `--tsconfig` flag (opt-in, like oxlint)

---

## High-Priority Gaps

### 3. Multi-File / Cross-File Analysis

**What's missing:** Module graph construction, cross-file import resolution.

**Why it matters:** Rules like `import/no-cycle`, `import/no-unresolved`, and
`no-barrel-file` require resolving imports across the project. These are among the
most requested ESLint rules.

**Oxlint:** Builds a project-wide module graph, resolves imports, handles
`tsconfig.paths`. Ran `import/no-cycle` over 126K files in 7 seconds at Airbnb.

**Recommended approach:**
- Build module graph in a pre-pass before per-file linting
- Share resolution results across rules via `LintContext`
- Start with `no-cycle`, `no-unresolved`, `no-self-import`

### 4. Shareable / Extendable Configs

**What's missing:** `extends` field in `starlint.toml`, config presets, named configs.

**Why it matters:** Teams need to share lint configurations across repos. Companies
publish org-wide configs (e.g., `@airbnb/eslint-config`). Without `extends`, every
repo must duplicate configuration.

**ESLint:** Flat config supports `extends` arrays, imports from npm packages.

**Oxlint:** Supports `extends` in `.oxlintrc.json`.

**Recommended approach:**
- Add `extends = ["./base.toml", "starlint:recommended"]` to config
- Support local file paths and built-in presets
- Consider npm/crate-based config packages later

### 5. Result Caching

**What's missing:** `--cache` flag to skip unchanged files.

**Why it matters:** Even with Rust performance, large monorepos (100K+ files) benefit
from caching. ESLint's cache cuts re-lint time by 50-80% on typical runs.

**ESLint:** `--cache` writes `.eslintcache` (JSON with file hashes + results).
`--cache-location` and `--cache-strategy` (metadata vs content) options.

**Oxlint:** No caching (relies on raw speed — typically fast enough).

**Recommended approach:**
- Hash file content + config + rule versions → cache key
- Store diagnostics per file in a binary cache file
- `--cache` / `--cache-location` / `--no-cache` flags
- Invalidate on config change or rule version bump

---

## Medium-Priority Gaps

### 6. Additional Output Formats

**Current:** Pretty, JSON (NDJSON), Compact, Count.

**Missing formats worth adding:**
- **GitHub Actions** (`::error file=...::message`) — enables inline PR annotations
- **GitLab CI** (Code Quality JSON) — enables MR code quality widgets
- **JUnit XML** — CI systems (Jenkins, Bitbucket) use this natively
- **SARIF** — GitHub Advanced Security, VS Code SARIF Viewer
- **Stylish** — ESLint's default format, familiar to most JS developers

### 7. Fix Dry Run

**What's missing:** `--fix-dry-run` to preview fixes without writing files.

**Why it matters:** Users want to audit fixes before applying them, especially in CI
or when running unfamiliar rules.

### 8. Processors

**What's missing:** Extract and lint JS/TS from non-JS files (Markdown, HTML, Vue SFCs).

**Why it matters:** Vue SFCs embed `<script>` blocks. Markdown docs contain code
fences. Without processors, these can't be linted.

**Note:** Starlint already has Vue plugin rules (18), but true SFC support would
require a processor that extracts `<script>` content.

### 9. Code Path Analysis

**What's missing:** Control flow graph (CFG) construction for rules like
`no-unreachable`, `no-fallthrough`, `consistent-return`.

**ESLint:** Provides `onCodePathStart`, `onCodePathEnd`, `onCodePathSegmentStart`
events to rules, enabling analysis of reachability and control flow.

**Recommended approach:**
- Build CFG during or after AST construction
- Expose via `LintContext` for rules that declare interest
- Start with `no-unreachable` and `no-fallthrough`

### 10. ESLint-Compatible JS Plugin API

**What's missing:** Ability to run existing ESLint plugins unmodified.

**Oxlint:** Alpha support for JS plugins with ESLint-compatible API.

**Starlint advantage:** WASM plugin system is more portable and sandboxed.
However, the massive ESLint plugin ecosystem (4,000+ packages) is JS-based.

**Recommended approach:** Consider a compatibility shim that wraps ESLint plugins
into WASM or runs them in a JS runtime subprocess. Low priority given WASM strategy.

---

## Low-Priority / Nice-to-Have

| Feature | Notes |
|---|---|
| **Config migration tool** | `eslint-to-starlint` converter for adoption |
| **Checkstyle output** | Legacy CI format, declining usage |
| **Config inspector** | Web UI for debugging config (ESLint has this) |
| **Editor config integration** | EditorConfig-aware formatting rules |
| **Monorepo config cascading** | Per-directory config overrides (ESLint flat config does this) |

---

## Starlint Unique Advantages

These are features where starlint leads:

1. **WASM plugin system** — Neither ESLint nor oxlint supports WASM plugins.
   Sandboxed, portable, language-agnostic plugin authoring.
2. **Higher rule count** — 718 rules vs oxlint's ~695 (and growing).
3. **Multi-pass autofix** — Up to 10 convergence passes vs single-pass in most linters.
4. **Built-in LSP** — Ships as part of the core, not a separate extension.
5. **TOML config** — Simpler, no JS execution required for config loading.
6. **Plugin feature gates** — Compile custom distributions with only needed plugins.
7. **Flat indexed AST** — JSON-serializable, no lifetimes, WASM-friendly.

---

## Recommended Prioritization

### Phase 1 — Adoption Blockers
1. Inline disable comments (`starlint-disable`)
2. Shareable configs (`extends`)

### Phase 2 — Competitive Parity
3. Result caching (`--cache`)
4. GitHub/GitLab/SARIF output formats
5. Fix dry run (`--fix-dry-run`)

### Phase 3 — Advanced Analysis
6. Type-aware linting (via tsgo integration)
7. Multi-file analysis (module graph)
8. Code path analysis

### Phase 4 — Ecosystem
9. Processors (Vue SFC, Markdown)
10. Config migration tools
11. ESLint plugin compatibility layer
