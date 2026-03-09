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
| `cargo llvm-cov nextest --workspace --fail-under-lines 90` | Enforce coverage floor (hard minimum) |
| `cargo test -p starlint_parser proptest` | Run parser property tests |
| `cargo test -p starlint_scope proptest` | Run scope property tests |
| `cargo fuzz run fuzz_parse` | Fuzz the parser (requires nightly + cargo-fuzz) |
| `cargo fuzz run fuzz_parse_scope` | Fuzz parser → scope pipeline |

### Coverage Policy

Two Codecov checks run on every PR (configured in `.codecov.yml`):

- **`codecov/project`**: Overall coverage must not decrease vs base commit (no regression)
- **`codecov/patch`**: New/changed lines must have ≥95% coverage

The 90% `--fail-under-lines` floor remains as a hard minimum in CI.

## Workflow

Before committing: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

## Architecture

```
crates/
  starlint_cli/              # CLI binary (clap, orchestration)
  starlint_core/             # Linter engine (file discovery, diagnostics, overrides)
  starlint_rule_framework/   # Rule authoring (LintRule, LintContext, Plugin, traversal, fix utils)
  starlint_config/           # Config file loading (starlint.toml)
  starlint_ast/              # Flat indexed AST types (NodeId-based)
  starlint_parser/           # Custom JS/TS/JSX/TSX parser → AstTree
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
  starlint_benches/        # Per-rule criterion benchmarks (cargo bench -p starlint_benches)
editors/
  vscode/                    # VS Code extension (language client)
wit/
  plugin.wit                 # WIT interface definition (plugin ABI)
```

### Crate Dependency Graph

```
starlint_cli → starlint_core, starlint_config, starlint_loader, starlint_lsp, starlint_plugin_sdk, tokio
starlint_lsp → starlint_core, starlint_config, starlint_loader, starlint_plugin_sdk, tower-lsp, tokio
starlint_loader → starlint_core, starlint_config, starlint_rule_framework, starlint_plugin_sdk, starlint_wasm_host (feature-gated), all starlint_plugin_* crates (feature-gated)
starlint_core → starlint_ast, starlint_parser, starlint_scope, starlint_plugin_sdk, starlint_config, starlint_rule_framework
starlint_rule_framework → starlint_ast, starlint_scope, starlint_plugin_sdk
starlint_plugin_* → starlint_rule_framework, starlint_plugin_sdk, starlint_ast
starlint_scope → starlint_ast
starlint_parser → starlint_ast
starlint_config → toml, serde
starlint_wasm_host → starlint_plugin_sdk, starlint_core, wasmtime
starlint_plugin_sdk → serde
```

### Data Flow

1. CLI args parsed (clap) → `Cli` struct
2. Config resolved (walk up dirs for `starlint.toml`)
3. `file_discovery` walks dirs, filters by extension → file list
4. Per file (parallel via rayon): `starlint_parser::parse()` → `AstTree`
5. If semantic rules active: `starlint_scope::build_scope_data(&tree)` → `ScopeData`
6. Dispatch to all plugins uniformly via `Plugin::lint_file(&FileContext)`
7. Merge all diagnostics → apply overrides → format (pretty/json/compact) → exit code

### Key Design Decisions

- **Modular Plugin architecture**: All 709 rules live in 9 independent plugin crates (`starlint_plugin_*`). Each exports `create_plugin() -> Box<dyn Plugin>` and `all_rules()`. Native and WASM plugins implement the same `Plugin` trait. The loader uses a feature-gated registry — users can compile custom distributions with only needed plugins. Config uses `[plugins]` — no distinction between built-in and external.
- **Rule framework separation**: `starlint_rule_framework` provides `LintRule`, `LintContext`, `Plugin`, `LintRulePlugin` adapter, AST traversal, and fix utilities. Plugin crates depend only on the framework (never the engine). The engine (`starlint_core`) depends on the framework but not on any plugin crate.
- **Custom parser + flat AST**: `starlint_parser` produces a `NodeId`-indexed `AstTree` — no arena allocation, no lifetime constraints
- **Lightweight scope analysis**: `starlint_scope` builds scope tree, symbol table, and reference tracking in two passes over `AstTree`
- **Single-pass traversal**: Native rules receive `AstNodeType` via type-filtered dispatch inside `LintRulePlugin` — miss is free
- **Interest-based filtering**: WASM v1 plugins declare which node types they need
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

This project uses [beads](https://github.com/steveyegge/beads) (`bd`) for ALL issue tracking. Do NOT use markdown TODOs, task lists, or other tracking methods.

### Workflow
1. **Start of session**: `bd prime` runs automatically via hook
2. **Find ready work**: `bd ready --json`
3. **Claim a task**: `bd update <id> --status in_progress --claim --json`
4. **Create new issues**: `bd create "Title" --description "Details" -t <type> -p <priority> --json`
5. **Discover new work?** Link it: `bd create "Found bug" --description "Details" -p 1 --deps discovered-from:<parent-id> --json`
6. **Close completed work**: `bd close <id> --reason "summary" --json`
7. **Before ending session**: `bd sync`

### Issue Types & Priorities

Types: `bug`, `feature`, `task`, `epic`, `chore`

Priorities: `0` critical, `1` high, `2` medium (default), `3` low, `4` backlog

### Rules
- Always use `--json` flag for machine-readable output
- Never use `bd edit` (interactive editor)
- Include issue IDs in commit messages: `git commit -m "Fix X (bd-abc)"`
- Link discovered work with `discovered-from` dependencies
- Check `bd ready` before asking "what should I work on?"
- bd auto-syncs to `.beads/issues.jsonl` — no manual export needed

## Session Completion

Work is NOT complete until `git push` succeeds.

1. **File issues** for remaining work
2. **Run quality gates** (if code changed): `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
3. **Update issue status**: close finished work, update in-progress items
4. **Push**: `git pull --rebase && bd sync && git push`
5. **Verify**: `git status` must show "up to date with origin"
