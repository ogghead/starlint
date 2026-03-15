# starlint

A fast, Rust-based JavaScript/TypeScript linter with first-class WASM plugin support.

## Features

- **Fast**: Hand-written recursive descent parser, flat indexed AST, single-pass traversal, streaming output
- **709 rules**: Covers JS, TS, React, Next.js, Vue, Jest, Vitest, JSDoc, Storybook, and more
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
| express | 141 | **17ms (12 MB)** | 85ms (112 MB) | 1.53s (255 MB) |
| date-fns | 1562 | **88ms (12 MB)** | 91ms (112 MB) | 4.75s (440 MB) |
| grafana | 6259 | 494ms (36 MB) | **362ms (137 MB)** | 32.61s (552 MB) |
<details>
<summary>All rules (~630-710 rules per tool)</summary>

| Corpus | Files | starlint | oxlint | eslint |
|--------|------:|----------|--------|--------|
| express | 141 | **59ms (18 MB)** | 279ms (133 MB) | 13.63s (746 MB) |
</details>

*Last updated: 2026-03-15. Benchmarked with [hyperfine](https://github.com/sharkdp/hyperfine) (3 warmup, 10+ runs).*
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

## Rules

709 rules organized into 9 plugin bundles:

| Category | Rules | Plugin |
|----------|------:|--------|
| General (JS/TS) | 326 | `core` |
| TypeScript | 99 | `typescript` |
| React | 55 | `react` |
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

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for architecture, development workflow, and how to add rules.

## License

MIT
