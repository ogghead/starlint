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

Full architecture details, crate dependency graph, and design decisions are in [CONTRIBUTING.md](CONTRIBUTING.md).

Key crates (quick reference):
- `starlint_cli` — CLI binary
- `starlint_core` — engine (file discovery, diagnostics, overrides)
- `starlint_rule_framework` — rule authoring (`LintRule`, `LintContext`, `Plugin`, traversal, fix utils)
- `starlint_parser` — custom JS/TS/JSX/TSX parser → `AstTree`
- `starlint_scope` — scope analysis (symbols, references, scopes)
- `starlint_plugin_sdk` — shared wire types (`Diagnostic`, `RuleMeta`, `Span`)
- `starlint_loader` — plugin loader (unified registry + WASM, feature-gated)
- `starlint_config` — config file loading (`starlint.toml`)
- `starlint_ast` — flat indexed AST types (`NodeId`-based)
- `starlint_lsp` — LSP server (tower-lsp, diagnostics, code actions)
- `starlint_wasm_host` — WASM runtime (wasmtime, bridge)
- `starlint_plugin_*` — 9 plugin crates (710 rules total), each exports `create_plugin()` and `all_rules()`
- `starlint_benches` — per-rule criterion benchmarks (`cargo bench -p starlint_benches`)

Data flow: CLI args → config → plugin loading → file discovery → per-file parallel (parse → scope if needed → dispatch to plugins) → diagnostics → overrides → output → optional fix passes

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
