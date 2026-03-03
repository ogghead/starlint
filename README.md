# starlint

A fast, Rust-based JavaScript/TypeScript linter with first-class WASM plugin support.

## Features

- **Fast**: Built on [oxc](https://oxc.rs) for parsing, with single-pass AST traversal
- **WASM Plugins**: Write lint rules in Rust (or any language targeting WASM) using the Component Model
- **Native Rules**: High-performance rules that operate directly on the oxc AST
- **Parallel**: File-level parallelism via rayon
- **Configurable**: `starlint.toml` with rule severity overrides and file pattern overrides

## Benchmarks

<!-- BENCHMARKS_START -->
Compared against [oxlint](https://oxc.rs) and [eslint](https://eslint.org) on real-world codebases with 20 equivalent lint rules.

| Corpus | Files | starlint | oxlint | eslint |
|--------|------:|----------|--------|--------|
| express | 141 | **24ms (12 MB)** | 107ms (111 MB) | 1.48s (243 MB) |
| date-fns | 1562 | **83ms (11 MB)** | 90ms (113 MB) | 4.69s (450 MB) |
| grafana | 6201 | 485ms (19 MB) | **385ms (136 MB)** | 31.82s (563 MB) |
<details>
<summary>Full defaults (all rules enabled per tool)</summary>

| Corpus | Files | starlint | oxlint | eslint |
|--------|------:|----------|--------|--------|
| express | 141 | 474ms (18 MB) | **73ms (106 MB)** | 1.83s (261 MB) |
| date-fns | 1562 | 1.73s (38 MB) | **139ms (111 MB)** | 5.88s (457 MB) |
| grafana | 6201 | 21.93s (296 MB) | **690ms (153 MB)** | 5.07s (481 MB) |
</details>

*Last updated: 2026-03-03. Benchmarked with [hyperfine](https://github.com/sharkdp/hyperfine) (3 warmup, 10+ runs).*
<!-- BENCHMARKS_END -->

## Status

**Early development** — the framework is being built. The first plugin (Storybook rules) will be ported from [oxlint-plugin-storybook](https://github.com/ogghead/oxlint-plugin-storybook).

## Quick Start

```bash
# Build
cargo build --workspace

# Run
cargo run -- .

# Initialize config
cargo run -- init
```

## Configuration

Create a `starlint.toml` in your project root:

```toml
[settings]
threads = 0  # 0 = auto-detect

[[plugins]]
name = "storybook"
path = "./plugins/starlint-plugin-storybook.wasm"

[rules]
"no-debugger" = "error"
"storybook/default-exports" = "error"

[[overrides]]
files = ["**/*.stories.tsx"]
[overrides.rules]
"storybook/default-exports" = "error"
```

## Plugin Development

Plugins implement the WIT interface defined in `wit/plugin.wit`. See the `examples/` directory for sample plugins.

## License

MIT
