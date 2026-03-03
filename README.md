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
| express | 141 | **26ms (11 MB)** | 381ms (112 MB) | 1.82s (257 MB) |
| date-fns | 1562 | **85ms (11 MB)** | 376ms (112 MB) | 4.93s (468 MB) |
| grafana | 6201 | **485ms (18 MB)** | 1.02s (167 MB) | 31.89s (581 MB) |
<details>
<summary>Full defaults (all rules enabled per tool)</summary>

| Corpus | Files | starlint | oxlint | eslint |
|--------|------:|----------|--------|--------|
| express | 141 | 482ms (18 MB) | **328ms (108 MB)** | 2.02s (254 MB) |
| date-fns | 1562 | 1.71s (38 MB) | **392ms (108 MB)** | 6.17s (475 MB) |
| grafana | 6201 | 22.60s (295 MB) | **934ms (155 MB)** | 5.07s (515 MB) |
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
