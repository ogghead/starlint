# Contributing to starlint

Rust workspace (edition 2024, rustc 1.85+, stable channel).

## Getting Started

```bash
# Build everything
cargo build --workspace

# Run the pre-commit checks (same as CI)
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```

## Commands

| Command | Purpose |
|---------|---------|
| `cargo check --workspace` | Fast type-checking |
| `cargo build --workspace` | Compile all crates |
| `cargo test --workspace` | Run all tests |
| `cargo nextest run --workspace` | Run tests with better output (preferred) |
| `cargo clippy --workspace --all-targets -- -D warnings` | Lint (warnings = errors) |
| `cargo fmt --all` | Format all code |
| `cargo fmt --all -- --check` | Check formatting |
| `cargo deny check` | Audit dependencies |
| `cargo machete` | Check for unused dependencies |
| `cargo llvm-cov nextest --workspace` | Coverage (text summary) |
| `cargo llvm-cov nextest --workspace --html --open` | Coverage in browser |
| `cargo llvm-cov nextest --workspace --fail-under-lines 90` | Enforce coverage floor |
| `cargo bench -p starlint_benches` | Run per-rule benchmarks |
| `cargo test -p starlint_parser proptest` | Run parser property tests |
| `cargo test -p starlint_scope proptest` | Run scope property tests |
| `cargo fuzz run fuzz_parse` | Fuzz the parser (requires nightly + cargo-fuzz) |

## CI Pipeline

CI runs on every push to `master` and every pull request:

1. **Fmt / Clippy / Machete** — formatting, lint warnings as errors, unused dependency check
2. **MSRV (1.85)** — `cargo check` on minimum supported Rust version
3. **Dependency audit** — `cargo deny` for license and vulnerability checks
4. **Test + Coverage** — full test suite with `cargo-llvm-cov`, enforces 90% floor

### Coverage Policy

Two Codecov checks run on every PR (configured in `.codecov.yml`):

- **`codecov/project`**: Overall coverage must not decrease vs base commit (no regression)
- **`codecov/patch`**: New/changed lines must have >=95% coverage

The 90% `--fail-under-lines` floor remains as a hard minimum in CI.

## Architecture

```
crates/
  starlint_cli/              # CLI binary (clap, orchestration)
  starlint_core/             # Linter engine (file discovery, diagnostics, overrides)
  starlint_rule_framework/   # Rule authoring (LintRule, LintContext, Plugin, traversal, fix utils)
  starlint_config/           # Config file loading (starlint.toml)
  starlint_ast/              # Flat indexed AST types (NodeId-based)
  starlint_parser/           # Custom JS/TS/JSX/TSX parser -> AstTree
  starlint_scope/            # Lightweight scope analysis (symbols, references, scopes)
  starlint_lsp/              # LSP server (tower-lsp, diagnostics, code actions)
  starlint_plugin_sdk/       # Shared wire types (Diagnostic, RuleMeta, Span)
  starlint_loader/           # Plugin loader (unified registry + WASM loading)
  starlint_wasm_host/        # WASM runtime (wasmtime, bridge)
  starlint_plugin_core/      # 326 core JS/TS rules
  starlint_plugin_react/     # 87 React + JSX a11y + perf rules
  starlint_plugin_typescript/ # 99 TypeScript rules
  starlint_plugin_testing/   # 71 Jest + Vitest rules
  starlint_plugin_modules/   # 55 import + node + promise rules
  starlint_plugin_nextjs/    # 21 Next.js rules
  starlint_plugin_vue/       # 17 Vue rules
  starlint_plugin_jsdoc/     # 18 JSDoc rules
  starlint_plugin_storybook/ # 15 Storybook rules
  starlint_benches/          # Per-rule criterion benchmarks
editors/
  vscode/                    # VS Code extension (language client)
wit/
  plugin.wit                 # WIT interface definition (plugin ABI)
```

### Crate Dependency Graph

```
starlint_cli -> starlint_core, starlint_config, starlint_loader, starlint_lsp, starlint_plugin_sdk
starlint_lsp -> starlint_core, starlint_config, starlint_loader, starlint_plugin_sdk
starlint_loader -> starlint_core, starlint_config, starlint_rule_framework, starlint_wasm_host (feature-gated), all starlint_plugin_* (feature-gated)
starlint_core -> starlint_ast, starlint_parser, starlint_scope, starlint_plugin_sdk, starlint_config, starlint_rule_framework
starlint_rule_framework -> starlint_ast, starlint_scope, starlint_plugin_sdk
starlint_plugin_* -> starlint_rule_framework, starlint_plugin_sdk, starlint_ast
starlint_scope -> starlint_ast
starlint_parser -> starlint_ast
starlint_wasm_host -> starlint_plugin_sdk, starlint_core, wasmtime
starlint_plugin_sdk -> serde
starlint_config -> toml, serde
```

### Data Flow

```
CLI args -> config resolution -> plugin loading (starlint_loader)
                                         |
                                         v
                              file discovery -> file list
                                         |
                           +-------------+-------------+
                           |    per-file (parallel)     |
                           |                            |
                           |  parse -> AstTree          |
                           |  scope analysis (if needed)|
                           |            |               |
                           |     +------+------+        |
                           |     |             |        |
                           |   native        WASM      |
                           |   plugins       plugins   |
                           |     |             |        |
                           |     +------+------+        |
                           |       diagnostics          |
                           +-------------+--------------+
                                         |
                           severity + file-pattern overrides
                                         |
                           stream to stdout (pretty/json/compact/count)
                                         |
                           optional fix passes -> exit code
```

### Key Design Decisions

- **Modular plugin architecture**: All 709 rules live in 9 independent plugin crates (`starlint_plugin_*`). Each exports `create_plugin() -> Box<dyn Plugin>` and `all_rules()`. Native and WASM plugins implement the same `Plugin` trait. The loader uses a feature-gated registry -- users can compile custom distributions with only needed plugins. Config uses `[plugins]` with no distinction between built-in and external.
- **Rule framework separation**: `starlint_rule_framework` provides `LintRule`, `LintContext`, `Plugin`, `LintRulePlugin` adapter, AST traversal, and fix utilities. Plugin crates depend only on the framework (never the engine). The engine (`starlint_core`) depends on the framework but not on any plugin crate.
- **Custom parser + flat AST**: `starlint_parser` produces a `NodeId`-indexed `AstTree` -- no arena allocation, no lifetime constraints. Sidesteps WIT's inability to express recursive types while enabling zero-copy traversal and JSON serialization for WASM plugins.
- **Lightweight scope analysis**: `starlint_scope` builds scope tree, symbol table, and reference tracking in two passes over `AstTree`. Only runs when a plugin requests it via `needs_scope_analysis()`. Used by 12 semantic rules.
- **Single-pass traversal**: Native rules receive `AstNodeType` via `enter_node` -- rules declare interest in specific node types, so non-matching rules are free.
- **Batched WASM calls**: One `lint-file` call per file per plugin (not per-node) to amortize serialization overhead.
- **WASM sandboxing**: Each plugin runs with fuel (10M instructions) and memory (16 MB) limits per file. Uses wasmtime's Winch baseline compiler.
- **Streaming output**: Diagnostics are written directly to stdout per-file via `BufWriter` -- no intermediate string buffering. `--format count` skips formatting entirely for maximum throughput.
- **Multi-pass fix convergence**: After applying fixes, files are re-linted and remaining fixable diagnostics are applied again (up to 10 passes), handling overlapping fixes.

## Adding a Native Lint Rule

1. Pick the plugin crate (e.g. `starlint_plugin_core` for general JS/TS rules).

2. Create `crates/starlint_plugin_<name>/src/rules/<rule_name>.rs`:

```rust
use starlint_ast::node::AstNode;
use starlint_ast::types::AstNodeType;
use starlint_plugin_sdk::diagnostic::Span;
use starlint_rule_framework::lint_rule::{LintContext, LintRule, RuleMeta};

/// Short description of what the rule checks.
pub struct MyRule;

impl LintRule for MyRule {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "my-rule".into(),
            category: "core".into(), // matches plugin name
            description: "Disallow X because Y".into(),
            fixable: false,
        }
    }

    fn interested_node_types(&self) -> &[AstNodeType] {
        &[AstNodeType::CallExpression]
    }

    fn enter_node(&self, node: &AstNode, ctx: &LintContext<'_>) {
        // Check the node, report diagnostics via ctx.report()
    }
}
```

3. Register in `crates/starlint_plugin_<name>/src/rules/mod.rs` and the crate's `all_rules()`.

4. Add tests using the `lint_source` test helper:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use starlint_rule_framework::lint_rule::lint_source;

    #[test]
    fn test_my_rule_catches_violation() {
        let diagnostics = lint_source("offending code here", "test.js", MyRule);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn test_my_rule_allows_valid_code() {
        let diagnostics = lint_source("valid code here", "test.js", MyRule);
        assert!(diagnostics.is_empty());
    }
}
```

5. Run `cargo test -p starlint_plugin_<name>` to verify.

## WASM Plugin Development

WASM plugins implement the WIT interface defined in [`wit/plugin.wit`](wit/plugin.wit):

```wit
interface plugin {
    get-rules: func() -> list<rule-meta>;
    get-file-patterns: func() -> list<string>;
    configure: func(config: plugin-config) -> list<string>;
    lint-file: func(file: file-context, tree: serialized-ast-tree) -> list<lint-diagnostic>;
}
```

Plugins receive the full AST as JSON bytes and return diagnostics with optional auto-fixes. See [`examples/plugins/`](examples/plugins/) for working examples.

Build a plugin:

```bash
cd examples/plugins/starlint-plugin-example
cargo build --target wasm32-unknown-unknown --release
wasm-tools component new \
  target/wasm32-unknown-unknown/release/starlint_plugin_example.wasm \
  -o starlint-plugin-example.wasm
```

Load it via config:

```toml
[plugins]
custom = { path = "./starlint-plugin-example.wasm" }
```

## Rust Conventions

### Error Handling

- Use `miette` for all errors: `#[derive(Diagnostic, Error)]`
- Never use `.unwrap()`, `.expect()`, or indexing -- these are denied by lint
- Use `?` for propagation, `.map_err()` to adapt errors

### Lint Config

- All crates inherit `[workspace.lints]` via `[lints] workspace = true`
- Clippy: pedantic + nursery + restriction lints enabled
- Rustdoc: `broken_intra_doc_links`, `bare_urls`, `invalid_html_tags`, `invalid_rust_codeblocks`, `missing_crate_level_docs`, `unescaped_backticks` all denied
- `#[allow(clippy::struct_excessive_bools)]` for CLI args and node interest flags
- `#[allow(clippy::let_underscore_must_use)]` for infallible `writeln!` to String

### Testing

- Unit tests: `#[cfg(test)] mod tests` at bottom of each file
- Integration tests: `crates/starlint_core/tests/`
- Fixtures: `crates/starlint_core/tests/fixtures/{valid,invalid}/`
- Property tests: `starlint_parser` and `starlint_scope` use proptest
- Fuzz tests: `cargo fuzz run fuzz_parse` (requires nightly)

## License

MIT
