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
| express | 141 | **12ms (13 MB)** | 118ms (114 MB) | 1.56s (251 MB) |
| date-fns | 1562 | **50ms (12 MB)** | 83ms (111 MB) | 4.82s (455 MB) |
| grafana | 6201 | **278ms (18 MB)** | 368ms (137 MB) | 34.14s (545 MB) |
<details>
<summary>Full defaults (all rules enabled per tool)</summary>

| Corpus | Files | starlint | oxlint | eslint |
|--------|------:|----------|--------|--------|
| express | 141 | 90ms (19 MB) | **69ms (106 MB)** | 1.77s (261 MB) |
| date-fns | 1562 | 483ms (39 MB) | **123ms (111 MB)** | 6.20s (464 MB) |
| grafana | 6201 | 3.40s (295 MB) | **639ms (151 MB)** | 4.83s (515 MB) |
</details>

*Last updated: 2026-03-05. Benchmarked with [hyperfine](https://github.com/sharkdp/hyperfine) (3 warmup, 10+ runs).*
<!-- BENCHMARKS_END -->

## Flamegraph

Profiling starlint on the [Grafana](https://github.com/grafana/grafana) codebase (~6k files) with default rules:

[![Flamegraph](https://raw.githubusercontent.com/ogghead/starlint/flamegraph-assets/flamegraph.svg)](https://raw.githubusercontent.com/ogghead/starlint/flamegraph-assets/flamegraph.svg)

<sub>Click for interactive view. Auto-generated on each push to master.</sub>

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
