#!/usr/bin/env bash
# Setup script for starlint benchmarks.
# Clones test corpora, installs eslint/oxlint, builds starlint release binary.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CORPORA_DIR="$SCRIPT_DIR/corpora"

# ── Prerequisites ──────────────────────────────────────────────────────────

check_cmd() {
    if ! command -v "$1" &>/dev/null; then
        echo "ERROR: $1 is required but not found. $2" >&2
        exit 1
    fi
}

check_cmd git "Install git"
check_cmd node "Install Node.js (>=18)"
check_cmd npm "Install Node.js (>=18)"
check_cmd cargo "Install Rust via rustup"
check_cmd hyperfine "Install: cargo install hyperfine  OR  brew install hyperfine"
check_cmd jq "Install: sudo pacman -S jq  OR  brew install jq"

echo "All prerequisites found."

# ── Clone corpora ──────────────────────────────────────────────────────────

mkdir -p "$CORPORA_DIR"

clone_if_missing() {
    local name="$1" repo="$2" subdir="${3:-}"
    local target="$CORPORA_DIR/$name"

    if [ -d "$target" ]; then
        echo "Corpus '$name' already exists, skipping."
        return
    fi

    echo "Cloning $repo (depth 1)..."
    if [ -n "$subdir" ]; then
        # Sparse checkout for large repos — only fetch the subdirectory we need
        git clone --depth 1 --filter=blob:none --sparse "$repo" "$target"
        (cd "$target" && git sparse-checkout set "$subdir")
    else
        git clone --depth 1 "$repo" "$target"
    fi
    echo "  → $target"
}

clone_if_missing "express"  "https://github.com/expressjs/express.git"
clone_if_missing "date-fns" "https://github.com/date-fns/date-fns.git"
clone_if_missing "grafana"  "https://github.com/grafana/grafana.git" "public/app"

# ── Install eslint & oxlint ───────────────────────────────────────────────

echo "Installing eslint and oxlint..."
(cd "$SCRIPT_DIR" && npm install --save-dev eslint @eslint/js @typescript-eslint/parser oxlint)

# ── Build starlint ────────────────────────────────────────────────────────

JOBS_FLAG=""
if [ -n "${CARGO_BUILD_JOBS:-}" ]; then
    JOBS_FLAG="-j $CARGO_BUILD_JOBS"
    echo "Building starlint (release, $CARGO_BUILD_JOBS jobs)..."
else
    echo "Building starlint (release)..."
fi
# shellcheck disable=SC2086
(cd "$REPO_ROOT" && cargo build --release $JOBS_FLAG)

STARLINT_BIN="$REPO_ROOT/target/release/starlint"
if [ ! -f "$STARLINT_BIN" ]; then
    echo "ERROR: starlint binary not found at $STARLINT_BIN" >&2
    exit 1
fi

echo ""
echo "Setup complete!"
echo "  Corpora:  $CORPORA_DIR/{express,date-fns,grafana}"
echo "  eslint:   $(npx --prefix "$SCRIPT_DIR" eslint --version 2>/dev/null || echo 'installed')"
echo "  oxlint:   $(npx --prefix "$SCRIPT_DIR" oxlint --version 2>/dev/null || echo 'installed')"
echo "  starlint: $STARLINT_BIN"
echo ""
echo "Run:  ./benches/run.sh"
