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

echo "Installing eslint, oxlint, and eslint plugins..."
(cd "$SCRIPT_DIR" && npm install --save-dev \
    eslint@9 @eslint/js @typescript-eslint/parser oxlint \
    typescript typescript-eslint \
    eslint-plugin-react eslint-plugin-react-hooks \
    eslint-plugin-jsx-a11y eslint-plugin-import-x \
    eslint-plugin-jest eslint-plugin-promise \
    eslint-plugin-n eslint-plugin-jsdoc \
    eslint-plugin-vue vue-eslint-parser \
    --legacy-peer-deps)

# ── Build starlint ────────────────────────────────────────────────────────

# CARGO_PROFILE: "release" (default) or "bench" (faster compile, no LTO)
PROFILE="${CARGO_PROFILE:-release}"
PROFILE_FLAG="--profile $PROFILE"

JOBS_FLAG=""
if [ -n "${CARGO_BUILD_JOBS:-}" ]; then
    JOBS_FLAG="-j $CARGO_BUILD_JOBS"
    echo "Building starlint (profile=$PROFILE, $CARGO_BUILD_JOBS jobs)..."
else
    echo "Building starlint (profile=$PROFILE)..."
fi
# shellcheck disable=SC2086
(cd "$REPO_ROOT" && cargo build $PROFILE_FLAG $JOBS_FLAG)

# Bench profile outputs to target/bench/, release to target/release/
if [ "$PROFILE" = "release" ]; then
    STARLINT_BIN="$REPO_ROOT/target/release/starlint"
else
    STARLINT_BIN="$REPO_ROOT/target/$PROFILE/starlint"
fi

if [ ! -f "$STARLINT_BIN" ]; then
    echo "ERROR: starlint binary not found at $STARLINT_BIN" >&2
    exit 1
fi

# Export for run.sh to pick up
export STARLINT_BIN

echo ""
echo "Setup complete!"
echo "  Corpora:  $CORPORA_DIR/{express,date-fns,grafana}"
echo "  eslint:   $("$SCRIPT_DIR/node_modules/.bin/eslint" --version 2>/dev/null || echo 'installed')"
echo "  oxlint:   $("$SCRIPT_DIR/node_modules/.bin/oxlint" --version 2>/dev/null || echo 'installed')"
echo "  starlint: $STARLINT_BIN"
echo ""
echo "Run:  ./benches/run.sh"
