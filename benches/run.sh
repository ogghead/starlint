#!/usr/bin/env bash
# Run benchmarks comparing eslint, oxlint, and starlint.
#
# Usage:
#   ./benches/run.sh                             # Run all scenarios x all corpora
#   ./benches/run.sh --corpus express             # Single corpus
#   ./benches/run.sh --scenario equivalent        # Single scenario
#   ./benches/run.sh --corpus express --scenario equivalent
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CORPORA_DIR="$SCRIPT_DIR/corpora"
CONFIGS_DIR="$SCRIPT_DIR/configs"
RESULTS_DIR="$SCRIPT_DIR/results"
# Derive binary path from CARGO_PROFILE (matches setup.sh logic)
if [ -n "${STARLINT_BIN:-}" ]; then
    : # explicitly set, use as-is
elif [ -n "${CARGO_PROFILE:-}" ] && [ "$CARGO_PROFILE" != "release" ]; then
    STARLINT_BIN="$REPO_ROOT/target/$CARGO_PROFILE/starlint"
else
    STARLINT_BIN="$REPO_ROOT/target/release/starlint"
fi

OXLINT_BIN="$SCRIPT_DIR/node_modules/.bin/oxlint"
ESLINT_BIN="$SCRIPT_DIR/node_modules/.bin/eslint"

WARMUP=3
MIN_RUNS=10

# ── Parse args ─────────────────────────────────────────────────────────────

CORPUS_FILTER=""
SCENARIO_FILTER=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --corpus)   CORPUS_FILTER="$2"; shift 2 ;;
        --scenario) SCENARIO_FILTER="$2"; shift 2 ;;
        --warmup)   WARMUP="$2"; shift 2 ;;
        --runs)     MIN_RUNS="$2"; shift 2 ;;
        -h|--help)
            echo "Usage: $0 [--corpus NAME] [--scenario NAME] [--warmup N] [--runs N]"
            echo "  Corpora:   express, date-fns, grafana"
            echo "  Scenarios: equivalent, default"
            exit 0
            ;;
        *) echo "Unknown arg: $1" >&2; exit 1 ;;
    esac
done

# ── Validation ─────────────────────────────────────────────────────────────

if [ ! -f "$STARLINT_BIN" ]; then
    echo "ERROR: starlint binary not found. Run ./benches/setup.sh first." >&2
    exit 1
fi

if [ ! -d "$CORPORA_DIR/express" ]; then
    echo "ERROR: corpora not found. Run ./benches/setup.sh first." >&2
    exit 1
fi

if [ ! -f "$OXLINT_BIN" ]; then
    echo "ERROR: oxlint not found at $OXLINT_BIN. Run ./benches/setup.sh first." >&2
    exit 1
fi

if [ ! -f "$ESLINT_BIN" ]; then
    echo "ERROR: eslint not found at $ESLINT_BIN. Run ./benches/setup.sh first." >&2
    exit 1
fi

mkdir -p "$RESULTS_DIR"

# ── Helpers ────────────────────────────────────────────────────────────────

corpus_path() {
    local name="$1"
    case "$name" in
        express)  echo "$CORPORA_DIR/express" ;;
        date-fns) echo "$CORPORA_DIR/date-fns" ;;
        grafana)  echo "$CORPORA_DIR/grafana/public/app" ;;
        *) echo "Unknown corpus: $name" >&2; exit 1 ;;
    esac
}

count_files() {
    local dir="$1"
    find "$dir" -type f \( -name '*.js' -o -name '*.jsx' -o -name '*.ts' -o -name '*.tsx' -o -name '*.mjs' -o -name '*.cjs' \) | wc -l
}

measure_memory() {
    local label="$1" cmd="$2" outfile="$3"
    local mem_kb=""
    if command -v /usr/bin/time &>/dev/null; then
        # GNU time: most accurate
        mem_kb=$(/usr/bin/time -v bash -c "$cmd >/dev/null 2>&1" 2>&1 | grep "Maximum resident" | awk '{print $NF}') || true
    elif command -v python3 &>/dev/null; then
        # Fallback: Python resource.getrusage (ru_maxrss of child process)
        mem_kb=$(python3 -c "
import subprocess, resource
resource.getrusage(resource.RUSAGE_CHILDREN)  # baseline
subprocess.run('$cmd', shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
print(int(resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss))
" 2>/dev/null) || true
    fi
    if [ -n "$mem_kb" ] && [ "$mem_kb" != "0" ]; then
        echo "    $label peak RSS: ${mem_kb} KB"
        echo "$label $mem_kb" >> "$outfile"
    else
        echo "    $label peak RSS: (skipped — install GNU time or python3)"
    fi
}

# Run hyperfine + memory for a trio of linter commands.
run_bench() {
    local name="$1"
    local starlint_cmd="$2"
    local oxlint_cmd="$3"
    local eslint_cmd="$4"

    echo ""
    echo "━━━ Benchmark: $name ━━━"

    hyperfine \
        --warmup "$WARMUP" \
        --min-runs "$MIN_RUNS" \
        --export-json "$RESULTS_DIR/${name}.json" \
        --export-markdown "$RESULTS_DIR/${name}.md" \
        -n "starlint" "$starlint_cmd" \
        -n "oxlint"   "$oxlint_cmd" \
        -n "eslint"   "$eslint_cmd"

    echo "  Measuring peak memory..."
    rm -f "$RESULTS_DIR/${name}.mem"
    measure_memory "starlint" "$starlint_cmd" "$RESULTS_DIR/${name}.mem"
    measure_memory "oxlint"   "$oxlint_cmd"   "$RESULTS_DIR/${name}.mem"
    measure_memory "eslint"   "$eslint_cmd"   "$RESULTS_DIR/${name}.mem"
}

should_run_corpus() {
    [ -z "$CORPUS_FILTER" ] || [ "$CORPUS_FILTER" = "$1" ]
}

should_run_scenario() {
    [ -z "$SCENARIO_FILTER" ] || [ "$SCENARIO_FILTER" = "$1" ]
}

# ── Scenario: Equivalent rules (20 rules, fair comparison) ────────────────

run_equivalent() {
    local corpus_name="$1"
    local cpath nfiles
    cpath="$(corpus_path "$corpus_name")"
    nfiles="$(count_files "$cpath")"

    echo "  Corpus: $corpus_name ($nfiles files)"
    echo "$corpus_name $nfiles" >> "$RESULTS_DIR/file-counts.txt"

    run_bench "equivalent-${corpus_name}" \
        "$STARLINT_BIN --format count --config $CONFIGS_DIR/starlint-equivalent.toml $cpath || true" \
        "$OXLINT_BIN --config $CONFIGS_DIR/oxlint-equivalent.json $cpath >/dev/null || true" \
        "$ESLINT_BIN --no-config-lookup -c $CONFIGS_DIR/eslint-equivalent.config.mjs $cpath >/dev/null || true"
}

# ── Scenario: Full defaults ───────────────────────────────────────────────

run_default() {
    local corpus_name="$1"
    local cpath nfiles
    cpath="$(corpus_path "$corpus_name")"
    nfiles="$(count_files "$cpath")"

    echo "  Corpus: $corpus_name ($nfiles files)"

    run_bench "default-${corpus_name}" \
        "$STARLINT_BIN --format count --config $CONFIGS_DIR/starlint-default.toml $cpath || true" \
        "$OXLINT_BIN $cpath >/dev/null || true" \
        "$ESLINT_BIN --no-config-lookup -c $CONFIGS_DIR/eslint-default.config.mjs $cpath >/dev/null || true"
}

# ── Main ───────────────────────────────────────────────────────────────────

echo "╔══════════════════════════════════════════════════════════╗"
echo "║          starlint vs oxlint vs eslint benchmarks        ║"
echo "╠══════════════════════════════════════════════════════════╣"
echo "║  Warmup: $WARMUP    Min runs: $MIN_RUNS                        ║"
echo "╚══════════════════════════════════════════════════════════╝"

rm -f "$RESULTS_DIR/file-counts.txt"

CORPORA=(express date-fns grafana)

if should_run_scenario "equivalent"; then
    echo ""
    echo "▶ Scenario: Equivalent Rules (20 rules)"
    for c in "${CORPORA[@]}"; do
        if should_run_corpus "$c"; then
            run_equivalent "$c"
        fi
    done
fi

if should_run_scenario "default"; then
    echo ""
    echo "▶ Scenario: Full Defaults"
    for c in "${CORPORA[@]}"; do
        if should_run_corpus "$c"; then
            run_default "$c"
        fi
    done
fi

echo ""
echo "All benchmarks complete. Results in: $RESULTS_DIR/"
echo "Run ./benches/report.sh to generate a summary."
