# starlint

Rust workspace (edition 2024, rustc 1.85+, stable channel). A fast JS/TS linter with first-class WASM plugin support.

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
| `cargo llvm-cov nextest --workspace --fail-under-lines 60` | Enforce coverage threshold |

## Workflow

Before committing: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

## Architecture

```
crates/
  starlint_cli/           # CLI binary (clap, orchestration)
  starlint_core/          # Linter engine (parse, traverse, dispatch, diagnostics)
  starlint_config/        # Config file loading (starlint.toml)
  starlint_lsp/           # LSP server (tower-lsp, diagnostics, code actions)
  starlint_plugin_sdk/    # Shared types for plugins (no oxc dependency)
  starlint_wasm_host/     # WASM runtime (wasmtime, bridge, loader)
editors/
  vscode/                 # VS Code extension (language client)
wit/
  plugin.wit              # WIT interface definition (plugin ABI)
```

### Crate Dependency Graph

```
starlint_cli → starlint_core, starlint_config, starlint_wasm_host, starlint_lsp, tokio
starlint_lsp → starlint_core, starlint_config, starlint_wasm_host, tower-lsp, tokio
starlint_core → oxc_*, starlint_plugin_sdk, starlint_config
starlint_config → toml, serde
starlint_wasm_host → starlint_plugin_sdk, starlint_core, wasmtime, oxc_ast, oxc_ast_visit
starlint_plugin_sdk → serde (NO oxc dependency)
```

### Data Flow

1. CLI args parsed (clap) → `Cli` struct
2. Config resolved (walk up dirs for `starlint.toml`)
3. `file_discovery` walks dirs, filters by extension → file list
4. Per file (parallel via rayon): `Allocator` → `parser::parse_file()` → `Program`
5. Single-pass AST traversal → dispatch to native rules via `AstKind` match
6. Node collection → serialize for WASM plugins → call plugins
7. Merge all diagnostics → format (pretty/json/compact) → exit code

### Key Design Decisions

- **Dual rule system**: Native Rust rules (direct oxc AST) + WASM plugins (simplified stable AST)
- **Single-pass traversal**: Rules receive `AstKind` via `enter_node` — miss is free
- **Interest-based filtering**: WASM plugins declare which node types they need
- **Per-file `Allocator`**: oxc's arena allocation requires allocator to outlive AST
- **Parallel processing**: rayon for file-level parallelism
- **Batched WASM calls**: One `lint-file` call per file per plugin (not per-node)

## Rust Conventions

### Error Handling
- Use `miette` for all errors: `#[derive(Diagnostic, Error)]`
- NEVER use `.unwrap()`, `.expect()`, or indexing — these are denied by lint
- Use `?` for propagation, `.map_err()` to adapt errors

### Lint Config
- All crates inherit `[workspace.lints.clippy]` via `[lints] workspace = true`
- Pedantic + nursery + restriction lints enabled
- `#[allow(clippy::struct_excessive_bools)]` for CLI args and node interest flags
- `#[allow(clippy::let_underscore_must_use)]` for infallible `writeln!` to String
- `#[allow(unused_assignments)]` on error modules (thiserror 2.x false positive)

### Testing
- Unit tests: `#[cfg(test)] mod tests` at bottom of each file
- Integration tests: `crates/starlint_core/tests/`
- Fixtures: `crates/starlint_core/tests/fixtures/{valid,invalid}/`

## Task Tracking (Beads)

This project uses [beads](https://github.com/steveyegge/beads) (`bd`) for task tracking.

### Workflow
1. **Start of session**: `bd prime` runs automatically via hook
2. **Find ready work**: `bd ready --json`
3. **Claim a task**: `bd update <id> --status in_progress --claim --json`
4. **Create new issues**: `bd create "Title" --description "Details" -t <type> -p <priority> --json`
5. **Close completed work**: `bd close <id> --reason "summary" --json`
6. **Before ending session**: `bd sync`

### Rules
- Always use `--json` flag for machine-readable output
- Never use `bd edit` (interactive editor)
- Include issue IDs in commit messages: `git commit -m "Fix X (bd-abc)"`
