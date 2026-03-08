# starlint

A fast, Rust-based JavaScript/TypeScript linter with first-class WASM plugin support.

## Features

- **Fast**: Hand-written recursive descent parser, flat indexed AST, single-pass traversal
- **700+ rules**: Covers JS, TS, React, Next.js, Vue, Jest, Vitest, JSDoc, Storybook, and more
- **WASM plugins**: Write lint rules in Rust (or any language targeting WASM) using the Component Model
- **Auto-fix**: Safe and dangerous fix categories with `--fix` and `--fix-dangerous`
- **Parallel**: File-level parallelism via rayon
- **LSP**: Built-in language server for editor integration
- **Configurable**: `starlint.toml` with rule severity overrides and file pattern overrides

## Benchmarks

<!-- BENCHMARKS_START -->
Compared against [oxlint](https://oxc.rs) and [eslint](https://eslint.org) on real-world codebases with 20 equivalent lint rules.

| Corpus | Files | starlint | oxlint | eslint |
|--------|------:|----------|--------|--------|
| express | 141 | **17ms (12 MB)** | 109ms (112 MB) | 1.54s (257 MB) |
| date-fns | 1562 | **85ms (12 MB)** | 93ms (112 MB) | 4.87s (468 MB) |
| grafana | 6201 | 469ms (28 MB) | **399ms (137 MB)** | 32.35s (549 MB) |
<details>
<summary>Full defaults (all rules enabled per tool)</summary>

| Corpus | Files | starlint | oxlint | eslint |
|--------|------:|----------|--------|--------|
| express | 141 | 99ms (21 MB) | **72ms (108 MB)** | 1.85s (262 MB) |
| date-fns | 1562 | 565ms (67 MB) | **142ms (109 MB)** | 6.11s (470 MB) |
| grafana | 6201 | 3.89s (691 MB) | **688ms (153 MB)** | 5.00s (495 MB) |
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
  starlint_cli/         CLI binary (clap, orchestration)
  starlint_core/        Linter engine (traversal, rule dispatch, diagnostics)
  starlint_config/      Config file loading (starlint.toml)
  starlint_parser/      Hand-written JS/TS/JSX recursive descent parser
  starlint_ast/         Flat indexed AST (no oxc dependency, serializable)
  starlint_scope/       Lightweight scope analysis (symbol table, scope tree)
  starlint_plugin_sdk/  Shared types for plugins (diagnostics, fixes, metadata)
  starlint_wasm_host/   WASM plugin host (wasmtime component model)
  starlint_lsp/         LSP server (tower-lsp)
editors/
  vscode/               VS Code extension
wit/
  plugin.wit            WIT interface for WASM plugins
examples/
  plugins/              10 example WASM plugins
```

### Data Flow

```
CLI args → config resolution → file discovery
                                    │
                        ┌───────────┴───────────┐
                        │   per-file (parallel)  │
                        │                        │
                        │  parse → AstTree       │
                        │  scope analysis (opt)  │
                        │         │              │
                        │    ┌────┴────┐         │
                        │    │         │         │
                        │  native   WASM         │
                        │  rules    plugins      │
                        │    │         │         │
                        │    └────┬────┘         │
                        │    diagnostics         │
                        └───────────┬────────────┘
                                    │
                        format (pretty/json/compact) → exit code
```

### Key Design Decisions

- **Flat indexed AST**: Nodes reference children by index. Sidesteps WIT's inability to express recursive types while enabling zero-copy traversal and JSON serialization for WASM plugins.
- **Single-pass traversal**: Native rules receive `AstNodeType` via `enter_node` — rules that don't match are free.
- **Unified Plugin trait**: Both native rules and WASM plugins implement a single `Plugin` trait, giving the engine one dispatch interface.
- **Batched WASM calls**: One `lint-file` call per file per plugin (not per-node) to amortize serialization overhead.
- **Opt-in scope analysis**: Only built when a plugin requests it via `needs_scope_analysis()`.
- **WASM resource limits**: Each plugin is sandboxed with fuel (10M instructions) and memory (16 MB) limits per file.

### Rule Categories

| Category | Rules | Plugin |
|----------|------:|--------|
| General (JS/TS) | ~328 | built-in |
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
