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
| express | 141 | **13ms (12 MB)** | 109ms (109 MB) | 1.47s (253 MB) |
| date-fns | 1562 | **69ms (11 MB)** | 92ms (110 MB) | 4.71s (465 MB) |
| grafana | 6201 | **306ms (19 MB)** | 391ms (136 MB) | 31.09s (561 MB) |
<details>
<summary>Full defaults (all rules enabled per tool)</summary>

| Corpus | Files | starlint | oxlint | eslint |
|--------|------:|----------|--------|--------|
| express | 141 | 96ms (18 MB) | **71ms (106 MB)** | 1.76s (253 MB) |
| date-fns | 1562 | 557ms (36 MB) | **141ms (108 MB)** | 5.84s (463 MB) |
| grafana | 6201 | 3.64s (274 MB) | **689ms (153 MB)** | 4.75s (512 MB) |
</details>

*Last updated: 2026-03-04. Benchmarked with [hyperfine](https://github.com/sharkdp/hyperfine) (3 warmup, 10+ runs).*
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
