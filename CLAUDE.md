# starlint

Rust workspace (edition 2024, rustc 1.85+, stable channel). A fast JS/TS linter with first-class WASM plugin support.

## Operating Principles (Non-Negotiable)

- **Correctness over cleverness**: Prefer boring, readable solutions that are easy to maintain.
- **Smallest change that works**: Minimize blast radius; don't refactor adjacent code unless it meaningfully reduces risk or complexity.
- **Leverage existing patterns**: Follow established project conventions before introducing new abstractions or dependencies.
- **Prove it works**: "Seems right" is not done. Validate with tests/build/lint and/or a reliable manual repro.
- **Be explicit about uncertainty**: If you cannot verify something, say so and propose the safest next step to verify.

## Agent Teams (Required)

**Always create a Claude Code agent team for any non-trivial task.** This is mandatory for tasks involving 2+ files, architectural decisions, bug fixes, or new features.

### When to Create a Team

- Any task touching multiple crates or files
- Bug reports (one agent reproduces, one investigates root cause, one reviews the fix)
- New rules or features (one agent implements, one writes tests, one validates coverage/lint)
- Refactoring across the workspace
- Performance investigations (one agent profiles, one explores alternatives)

### Team Structure Guidelines

- Assign each teammate **one focused objective** with a concrete deliverable
- Keep the main agent as coordinator — merge teammate outputs into actionable synthesis before coding
- Use teammates for: repo exploration, pattern discovery, test failure triage, dependency research, risk review
- Prefer 2-4 teammates; avoid spawning more than necessary

### Example Team Prompts

**Bug fix:**
> Create a team: one teammate to reproduce and isolate the bug, one to research similar patterns in the codebase, one to draft a regression test.

**New lint rule:**
> Create a team: one teammate to study existing rules in the target plugin for patterns, one to research the ESLint equivalent rule behavior, one to draft test fixtures.

**Cross-crate refactor:**
> Create a team: one teammate to map all call sites and dependencies, one to assess risk and identify affected tests, one to prototype the change in an isolated crate.

## Workflow Orchestration

### Plan Mode Default
- Enter plan mode for any non-trivial task (3+ steps, multi-file change, architectural decision).
- Include verification steps in the plan (not as an afterthought).
- If new information invalidates the plan: **stop**, update the plan, then continue.

### Incremental Delivery
- Prefer **thin vertical slices** over big-bang changes.
- Land work in small, verifiable increments: implement → test → verify → then expand.
- When feasible, keep changes behind feature flags or safe defaults.

### Verification Before "Done"
- Never mark complete without evidence: tests, lint/typecheck, build, or deterministic manual repro.
- Compare behavior baseline vs changed behavior when relevant.
- Ask: "Would a staff engineer approve this diff and the verification story?"

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
| `cargo bench -p starlint_benches` | Run per-rule criterion benchmarks |
| `cargo test -p starlint_parser proptest` | Run parser property tests |
| `cargo test -p starlint_scope proptest` | Run scope property tests |
| `cargo fuzz run fuzz_parse` | Fuzz the parser (requires nightly + cargo-fuzz) |
| `cargo fuzz run fuzz_parse_scope` | Fuzz parser → scope pipeline |

### Pre-Commit Quality Gate

Before committing: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

### Coverage Policy

Two Codecov checks run on every PR (configured in `.codecov.yml`):

- **`codecov/project`**: Overall coverage must not decrease vs base commit (no regression)
- **`codecov/patch`**: New/changed lines must have ≥95% coverage

The 90% `--fail-under-lines` floor remains as a hard minimum in CI.

## Architecture

Full architecture details, crate dependency graph, and design decisions are in [CONTRIBUTING.md](CONTRIBUTING.md).

### Crate Overview (21 crates)

**Core infrastructure:**
- `starlint_cli` — CLI binary (clap-based, subcommands: lint, fix, init, rules, lsp)
- `starlint_core` — engine (file discovery, diagnostics, overrides, parallel execution via rayon)
- `starlint_rule_framework` — rule authoring (`LintRule`, `LintContext`, `Plugin`, traversal, fix utils)
- `starlint_parser` — hand-written recursive descent JS/TS/JSX/TSX parser → `AstTree`
- `starlint_ast` — flat indexed AST types (`NodeId`-based, serde-compatible)
- `starlint_scope` — lightweight scope analysis (symbols, references, scopes via two-pass traversal)
- `starlint_plugin_sdk` — shared wire types (`Diagnostic`, `RuleMeta`, `Span`)
- `starlint_config` — config file loading (`starlint.toml`, toml + serde)
- `starlint_loader` — plugin loader (unified registry + WASM, feature-gated)
- `starlint_wasm_host` — WASM runtime (wasmtime 42, Component Model, fuel/memory limits)
- `starlint_lsp` — LSP server (tower-lsp, diagnostics, code actions)
- `starlint_benches` — per-rule criterion benchmarks (`cargo bench -p starlint_benches`)

**Plugin crates (9 plugins, 718 rules total):**

| Plugin | Rules | Scope |
|--------|------:|-------|
| `starlint_plugin_core` | 327 | General JS/TS: best practices, style, correctness |
| `starlint_plugin_typescript` | 100 | TypeScript-specific rules |
| `starlint_plugin_react` | 88 | React + JSX a11y + React Perf |
| `starlint_plugin_testing` | 72 | Jest + Vitest |
| `starlint_plugin_modules` | 56 | Import + Node + Promise rules |
| `starlint_plugin_nextjs` | 22 | Next.js framework |
| `starlint_plugin_jsdoc` | 19 | JSDoc comments |
| `starlint_plugin_vue` | 18 | Vue framework |
| `starlint_plugin_storybook` | 16 | Storybook |

Each plugin exports `create_plugin() -> Box<dyn Plugin>` and `all_rules()`.

### Data Flow

CLI args → config → plugin loading → file discovery → per-file parallel (parse → scope if needed → dispatch to plugins) → diagnostics → overrides → output → optional fix passes (up to 10 iterations)

### Crate Dependency Graph

```
starlint_cli → starlint_core, starlint_config, starlint_loader, starlint_lsp, starlint_plugin_sdk
starlint_lsp → starlint_core, starlint_config, starlint_loader, starlint_plugin_sdk
starlint_loader → starlint_config, starlint_rule_framework, starlint_wasm_host (opt), all plugins (opt)
starlint_core → starlint_ast, starlint_parser, starlint_scope, starlint_plugin_sdk, starlint_config, starlint_rule_framework
starlint_rule_framework → starlint_ast, starlint_scope, starlint_plugin_sdk
starlint_plugin_* → starlint_rule_framework, starlint_plugin_sdk, starlint_ast
starlint_scope → starlint_ast
starlint_parser → starlint_ast
starlint_wasm_host → starlint_plugin_sdk, starlint_core, wasmtime
```

### Key Design Decisions

1. **Custom parser** — hand-written recursive descent, zero-copy, produces flat indexed AST directly
2. **Flat indexed AST** — `NodeId`-based references (no arenas/lifetimes), JSON-serializable for WASM
3. **Single-pass traversal** — native rules declare interest via `interested_node_types()`, non-matching rules are free
4. **Batched WASM** — one `lint-file` call per file per plugin (amortizes serialization)
5. **Feature-gated loader** — compile custom distributions with only needed plugins
6. **File-level parallelism** — rayon-based, not AST-node-level
7. **Streaming output** — diagnostics written directly to stdout via `BufWriter`

## Rust Conventions

### Error Handling
- Use `miette` for all errors: `#[derive(Diagnostic, Error)]`
- NEVER use `.unwrap()`, `.expect()`, or indexing — these are denied by lint
- Use `?` for propagation, `.map_err()` to adapt errors

### Lint Config
- All crates inherit `[workspace.lints.clippy]` via `[lints] workspace = true`
- Pedantic + nursery + restriction lints enabled
- `unsafe_code = "forbid"` — no unsafe Rust anywhere
- Key denied lints: `unwrap_used`, `expect_used`, `indexing_slicing`, `panic`, `todo`, `dbg_macro`, `print_stdout`, `as_conversions`, `wildcard_imports`, `missing_docs_in_private_items`
- `#[allow(clippy::struct_excessive_bools)]` for CLI args and node interest flags
- `#[allow(clippy::let_underscore_must_use)]` for infallible `writeln!` to String
- `#[allow(unused_assignments)]` on error modules (thiserror 2.x false positive)

### Testing
- Unit tests: `#[cfg(test)] mod tests` at bottom of each file
- Integration tests: `crates/starlint_core/tests/`
- Fixtures: `crates/starlint_core/tests/fixtures/{valid,invalid}/`
- Snapshot testing: `insta` crate (YAML format)
- Property testing: `proptest` (parser and scope analysis)
- Rule testing: `lint_source()` helper from `starlint_rule_framework` (feature: `test-utils`)
- Add the smallest test that would have caught the bug
- Prefer unit tests for pure logic, integration tests for boundaries

### Adding a New Rule

1. Create rule file in the appropriate plugin crate (e.g., `crates/starlint_plugin_core/src/rules/`)
2. Implement `LintRule` trait: `meta()`, `interested_node_types()`, `run_on_node()`
3. Register in plugin's `all_rules()` function in `lib.rs`
4. Add tests using `lint_source()` helper
5. Ensure ≥95% coverage on new lines

### Dependency Discipline
- Do not add new dependencies unless the existing stack cannot solve it cleanly
- Prefer standard library / existing workspace utilities
- `cargo deny check` enforces license allowlist and vulnerability audit
- `cargo machete` catches unused dependencies

## Error Handling and Recovery

### Stop-the-Line Rule
If anything unexpected happens (test failures, build errors, behavior regressions):
- Stop adding features
- Preserve evidence (error output, repro steps)
- Return to diagnosis and re-plan

### Bug Fix Triage (Use in Order)
1. **Reproduce** reliably (test, script, or minimal steps)
2. **Localize** the failure (which crate/layer)
3. **Reduce** to a minimal failing case
4. **Fix** root cause (not symptoms)
5. **Guard** with regression coverage
6. **Verify** end-to-end for the original report

## Communication Guidelines

- Lead with outcome and impact, not process
- Reference concrete artifacts: file paths, command names, error messages
- Ask **exactly one** targeted question when blocked, with a recommended default
- State assumptions and constraints explicitly
- Always include the verification story: what you ran and the outcome
- Don't narrate every step; provide checkpoints when scope changes, risks appear, or verification fails

## CI Pipeline

Runs on every push to `master` and every PR:

1. **Fmt / Clippy / Machete** — formatting, lint warnings as errors, unused dependency check
2. **MSRV (1.85)** — `cargo check` on minimum supported Rust version
3. **Dependency audit** — `cargo deny` for license (MIT, Apache-2.0, BSD-*, ISC, MPL-2.0, Unicode, Zlib) and vulnerability checks
4. **Test + Coverage** — full test suite with `cargo-llvm-cov`, enforces 90% floor, Codecov upload
5. **Benchmarks** (PRs) — hyperfine comparison against oxlint/eslint on real-world corpora

## Git and Change Hygiene

- Keep commits atomic and describable; avoid "misc fixes" bundles
- Don't rewrite history unless explicitly requested
- Don't mix formatting-only changes with behavioral changes
- Treat generated files carefully: only commit them if the project expects it

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

## Definition of Done

A task is done when:
- Behavior matches acceptance criteria
- `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` passes
- New/changed lines have ≥95% coverage
- Risky changes have a rollback strategy (feature flag, config gating, or isolated commits)
- Code follows existing conventions and is readable
- A short verification story exists: what changed + how we know it works

## Session Completion

Work is NOT complete until `git push` succeeds.

1. **File issues** for remaining work
2. **Run quality gates** (if code changed): `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
3. **Update issue status**: close finished work, update in-progress items
4. **Push**: `git pull --rebase && bd sync && git push`
5. **Verify**: `git status` must show "up to date with origin"
