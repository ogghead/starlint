# Changelog

All notable changes to starlint are documented in this file.

## [0.2.0] - 2026-03-09

### Added

- **12 new lint rules** (709 total): `no-anonymous-default-export`, `no-dupe-keys`, `no-process-exit`, `no-restricted-imports`, `prefer-includes`, `prefer-promise-reject-errors`, `prefer-string-starts-ends-with`, `require-await` (core); `checked-requires-onchange-or-readonly`, `no-namespace`, `no-redundant-should-component-update` (react); `no-namespace` (typescript)
- **Per-rule benchmark crate** (`starlint_benches`): criterion benchmarks for all 709 rules with realistic fixtures (~300-800 lines of production-style JS/TS/TSX code)
- **CONTRIBUTING.md**: comprehensive contributor guide with architecture, crate dependency graph, native rule authoring guide, and development workflow
- **Rustdoc quality lints**: `broken_intra_doc_links`, `bare_urls`, `invalid_html_tags`, `invalid_rust_codeblocks`, `missing_crate_level_docs`, `unescaped_backticks` all enforced as deny

### Changed

- **Workspace version**: all 21 crates now share a single version from `[workspace.package]` (was scattered across individual Cargo.toml files)
- **Package names standardized**: all workspace crates use underscores (`starlint_plugin_core`) instead of mixed hyphens/underscores
- **README slimmed**: user-facing only (features, benchmarks, config, rules); architecture and dev workflow moved to CONTRIBUTING.md

### Performance

- **File path guards**: skip rules early when file extension doesn't match
- **AST parent walking**: O(1) parent lookups via pre-built index
- **HashSet lookups**: replace linear scans in hot paths
- **Content guards**: skip rules when source text can't possibly match
- **SmallVec dispatch**: avoid heap allocation for per-node rule lists
- **Pre-filtered dispatch table**: build per-file dispatch table based on active rules
- **Parser/scope optimizations**: faster tokenization and scope tree construction
- **Streaming output**: diagnostics written directly via BufWriter, `--format count` skips formatting entirely

### Fixed

- Flamegraph CI builds with proper symbol resolution
- Benchmark OOM in CI by splitting scenarios
- Deduplicated file counts in README generation
- Parser error recovery: clamp function span end to prevent `start > end`

## [0.1.5] - 2026-03-07

### Added

- All-rules benchmark scenario (~630-710 rules per tool)
- Pre-commit hook with CI-equivalent checks

### Changed

- Refactored all rules into 9 independent plugin crates
- Removed dead code and backward-compat re-exports from `starlint_core`
- Added `--format count` output mode for maximum throughput

## [0.1.0] - 2026-03-01

Initial release.

- 697 lint rules across 9 plugin bundles (core, react, typescript, testing, modules, nextjs, vue, jsdoc, storybook)
- Custom recursive descent JS/TS/JSX/TSX parser
- Flat indexed AST with NodeId-based traversal
- WASM plugin support via Component Model
- Auto-fix with `--fix` and `--fix-dangerous`
- LSP server with VS Code extension
- File-level parallelism via rayon
- `starlint.toml` configuration with overrides
