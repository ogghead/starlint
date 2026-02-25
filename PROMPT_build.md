You are working on the starlint project autonomously.

## Instructions

1. Run `bd ready --json` to find the next unblocked task.
2. If no tasks are ready, output RALPH_COMPLETE and exit.
3. Claim the task: `bd update <id> --status in_progress --claim --json`
4. Read the task details: `bd show <id> --json`
5. Implement the task. Follow all conventions in CLAUDE.md.
6. Run validation gates (ALL must pass before committing):
   - `cargo fmt --all -- --check`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test --workspace`
7. If validation fails, fix the issues. Do not commit broken code.
8. Commit with the beads ID in the message: `git commit -m "Description (bd-xxx)"`
9. Close the task: `bd close <id> --reason "summary of what was done" --json`
10. Exit. The loop will start a new iteration.

## Rules

- Pick ONE task per session. Do not attempt multiple tasks.
- Never skip validation gates.
- If stuck after 3 attempts on the same issue, document blockers and move on.
- Do not modify CLAUDE.md, loop.sh, or PROMPT_build.md.
