# starlint

A fast, Rust-based JavaScript/TypeScript linter with first-class WASM plugin support.

## Features

- **Fast**: Hand-written recursive descent parser, flat indexed AST, single-pass traversal, streaming output
- **696 rules**: Covers JS, TS, React, Next.js, Vue, Jest, Vitest, JSDoc, Storybook, and more
- **WASM plugins**: Write lint rules in Rust (or any language targeting WASM) using the Component Model
- **Auto-fix**: Safe and dangerous fix categories with `--fix` and `--fix-dangerous`, multi-pass convergence
- **Parallel**: File-level parallelism via rayon
- **LSP**: Built-in language server for editor integration (VS Code extension included)
- **Configurable**: `starlint.toml` with rule severity overrides, file pattern overrides, and per-plugin control

## Benchmarks

<!-- BENCHMARKS_START -->
Compared against [oxlint](https://oxc.rs) and [eslint](https://eslint.org) on real-world codebases with 20 equivalent lint rules.

| Corpus | Files | starlint | oxlint | eslint |
|--------|------:|----------|--------|--------|
| express | 141 | **16ms (12 MB)** | 113ms (112 MB) | 1.54s (254 MB) |
| date-fns | 1562 | **85ms (12 MB)** | 92ms (111 MB) | 4.74s (436 MB) |
| grafana | 6201 | 507ms (28 MB) | **393ms (136 MB)** | 31.14s (540 MB) |
<details>
<summary>Full defaults (all rules enabled per tool)</summary>

| Corpus | Files | starlint | oxlint | eslint |
|--------|------:|----------|--------|--------|
| express | 141 | 96ms (18 MB) | **73ms (108 MB)** | 1.81s (253 MB) |
| date-fns | 1562 | 546ms (45 MB) | **141ms (110 MB)** | 5.94s (477 MB) |
| grafana | 6201 | 3.58s (404 MB) | **685ms (152 MB)** | 4.80s (526 MB) |
</details>

*Last updated: 2026-03-08. Benchmarked with [hyperfine](https://github.com/sharkdp/hyperfine) (3 warmup, 10+ runs).*
<!-- BENCHMARKS_END -->

## Flamegraph

Profiling starlint on the [Grafana](https://github.com/grafana/grafana) codebase (~6k files) with default rules:

[![Flamegraph](https://raw.githubusercontent.com/ogghead/starlint/flamegraph-assets/flamegraph.svg)](https://raw.githubusercontent.com/ogghead/starlint/flamegraph-assets/flamegraph.svg)

<sub>Click for interactive view. Auto-generated on each push to master.</sub>

## Quick Start

```bash
# Build
cargo build --workspace

# Run on current directory
cargo run -- .

# Initialize config
cargo run -- init

# Apply auto-fixes
cargo run -- fix .

# Start LSP server
cargo run -- lsp

# List all available rules
cargo run -- rules
```

## Configuration

Create a `starlint.toml` in your project root:

```toml
[settings]
threads = 0  # 0 = auto-detect

# Enable/disable plugins (all enabled by default when section omitted)
[plugins]
core = true            # General JS/TS rules
react = true           # React + JSX A11y + React Perf
typescript = true      # TypeScript rules
testing = true         # Jest + Vitest
modules = true         # Import + Node + Promise
nextjs = true          # Next.js rules
vue = true             # Vue rules
jsdoc = true           # JSDoc rules
storybook = true       # Storybook rules
custom = { path = "./plugins/custom-plugin.wasm" }  # External WASM

# Per-rule severity
[rules]
"no-debugger" = "error"
"typescript/no-explicit-any" = "warn"
"no-var" = "off"

# File-pattern overrides
[[overrides]]
files = ["**/*.stories.tsx"]
[overrides.rules]
"storybook/default-exports" = "error"
```

## Architecture

```
crates/
  starlint_cli/         CLI binary (clap, orchestration, fix application)
  starlint_core/        Linter engine (traversal, rule dispatch, diagnostics, overrides)
  starlint_config/      Config file loading and resolution (starlint.toml)
  starlint_loader/      Unified plugin loader (resolves native + WASM from config)
  starlint_parser/      Hand-written JS/TS/JSX/TSX recursive descent parser
  starlint_ast/         Flat indexed AST (NodeId-based, no lifetimes, serializable)
  starlint_scope/       Lightweight scope analysis (symbol table, scope tree, references)
  starlint_plugin_sdk/  Shared types for plugins (rules, diagnostics, fixes, metadata)
  starlint_wasm_host/   WASM plugin host (wasmtime component model, sandboxed)
  starlint_lsp/         LSP server (tower-lsp, diagnostics, code actions)
editors/
  vscode/               VS Code extension (language client)
wit/
  plugin.wit            WIT interface definition for WASM plugins
examples/
  plugins/              Example WASM plugins
```

### Data Flow

```
CLI args → config resolution → plugin loading (starlint_loader)
                                         │
                                         ▼
                              file discovery → file list
                                         │
                           ┌─────────────┴─────────────┐
                           │    per-file (parallel)     │
                           │                            │
                           │  parse → AstTree           │
                           │  scope analysis (if needed)│
                           │            │               │
                           │     ┌──────┴──────┐        │
                           │     │             │        │
                           │   native        WASM      │
                           │   plugins       plugins   │
                           │     │             │        │
                           │     └──────┬──────┘        │
                           │       diagnostics          │
                           └─────────────┬──────────────┘
                                         │
                           severity + file-pattern overrides
                                         │
                           stream to stdout (pretty/json/compact/count)
                                         │
                           optional fix passes → exit code
```

### Key Design Decisions

- **Custom parser**: Hand-written recursive descent parser handles JS, TS, JSX, and TSX. No external parser dependency. Auto-detects language from file extension (`.js`, `.jsx`, `.ts`, `.tsx`, `.mjs`, `.cjs`, `.mts`, `.cts`).
- **Flat indexed AST**: Nodes reference children by `NodeId` index. No arena allocation, no lifetime constraints. Sidesteps WIT's inability to express recursive types while enabling zero-copy traversal and JSON serialization for WASM plugins.
- **Single-pass traversal**: Native rules receive `AstNodeType` via `enter_node` — rules declare interest in specific node types, so non-matching rules are free.
- **Unified Plugin trait**: Both native rules and WASM plugins implement a single `Plugin` trait. The engine has one dispatch interface; config doesn't distinguish between native and external plugins.
- **Unified loader**: `starlint_loader` resolves plugins from config — if a name matches the native registry, it wraps as `LintRulePlugin`; if a `path` is specified, it loads external WASM. One code path for CLI and LSP.
- **Batched WASM calls**: One `lint-file` call per file per plugin (not per-node) to amortize serialization overhead.
- **Opt-in scope analysis**: `starlint_scope` builds scope tree, symbol table, and reference tracking only when a plugin requests it via `needs_scope_analysis()`. Used by 12 semantic rules.
- **WASM sandboxing**: Each plugin runs with fuel (10M instructions) and memory (16 MB) limits per file. Uses wasmtime's Winch baseline compiler.
- **Streaming output**: Diagnostics are written directly to stdout per-file via `BufWriter` — no intermediate string buffering. `--format count` skips formatting entirely for maximum throughput.
- **Multi-pass fix convergence**: After applying fixes, files are re-linted and remaining fixable diagnostics are applied again (up to 10 passes), handling overlapping fixes.

### Rule Categories

696 rules organized into 9 native plugin bundles:

| Category | Rules | Plugin |
|----------|------:|--------|
| General (JS/TS) | 318 | `core` |
| TypeScript | 98 | `typescript` |
| React | 52 | `react` |
| JSX A11y | 31 | `react` |
| Jest | 54 | `testing` |
| Vitest | 17 | `testing` |
| Import | 33 | `modules` |
| Promise | 16 | `modules` |
| Next.js | 21 | `nextjs` |
| JSDoc | 18 | `jsdoc` |
| Vue | 17 | `vue` |
| Storybook | 15 | `storybook` |
| Node | 6 | `modules` |
| React Perf | 4 | `react` |

## Plugin Development

WASM plugins implement the WIT interface defined in [`wit/plugin.wit`](wit/plugin.wit):

```wit
interface plugin {
    get-rules: func() -> list<rule-meta>;
    get-file-patterns: func() -> list<string>;
    configure: func(config: plugin-config) -> list<string>;
    lint-file: func(file: file-context, tree: serialized-ast-tree) -> list<lint-diagnostic>;
}
```

Plugins receive the full AST as JSON bytes and return diagnostics with optional auto-fixes. See [`examples/plugins/`](examples/plugins/) for 10 working examples covering React, TypeScript, testing, and more.

Build a plugin:

```bash
cd examples/plugins/starlint-plugin-example
cargo build --target wasm32-unknown-unknown --release
wasm-tools component new \
  target/wasm32-unknown-unknown/release/starlint_plugin_example.wasm \
  -o starlint-plugin-example.wasm
```

## License

MIT
