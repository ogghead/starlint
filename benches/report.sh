#!/usr/bin/env bash
# Parse benchmark results and generate a markdown summary.
#
# Usage:
#   ./benches/report.sh             # Print to stdout
#   ./benches/report.sh > report.md # Save to file
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="$SCRIPT_DIR/results"

if [ ! -d "$RESULTS_DIR" ] || ! ls "$RESULTS_DIR"/*.json &>/dev/null; then
    echo "ERROR: No benchmark results found. Run ./benches/run.sh first." >&2
    exit 1
fi

# ── Helpers ────────────────────────────────────────────────────────────────

# Extract median time from hyperfine JSON for a given tool name
get_median() {
    local json_file="$1" tool_name="$2"
    jq -r ".results[] | select(.command | contains(\"$tool_name\")) | .median" "$json_file" 2>/dev/null || echo "N/A"
}

# Extract stddev from hyperfine JSON
get_stddev() {
    local json_file="$1" tool_name="$2"
    jq -r ".results[] | select(.command | contains(\"$tool_name\")) | .stddev" "$json_file" 2>/dev/null || echo "N/A"
}

# Format seconds with appropriate precision
fmt_time() {
    local val="$1"
    if [ "$val" = "N/A" ]; then
        echo "N/A"
        return
    fi
    # Use awk for float comparison and formatting
    awk "BEGIN { v=$val; if (v < 1) printf \"%.0fms\", v*1000; else printf \"%.2fs\", v }"
}

# Get memory from .mem file
get_memory_kb() {
    local mem_file="$1" tool_name="$2"
    if [ -f "$mem_file" ]; then
        grep "^$tool_name " "$mem_file" | awk '{print $2}' || echo "N/A"
    else
        echo "N/A"
    fi
}

# Format KB to human-readable
fmt_mem() {
    local kb="$1"
    if [ "$kb" = "N/A" ] || [ -z "$kb" ]; then
        echo "N/A"
        return
    fi
    awk "BEGIN { mb=$kb/1024; if (mb >= 1024) printf \"%.1f GB\", mb/1024; else printf \"%.0f MB\", mb }"
}

# Calculate speedup ratio
speedup() {
    local slow="$1" fast="$2"
    if [ "$slow" = "N/A" ] || [ "$fast" = "N/A" ]; then
        echo "N/A"
        return
    fi
    awk "BEGIN { if ($fast > 0) printf \"%.1fx\", $slow/$fast; else print \"N/A\" }"
}

# ── Report ─────────────────────────────────────────────────────────────────

echo "# Benchmark Results"
echo ""
echo "Generated: $(date -u '+%Y-%m-%d %H:%M UTC')"
echo ""

# Process each JSON result file
for json_file in "$RESULTS_DIR"/*.json; do
    basename="$(basename "$json_file" .json)"
    scenario="${basename%%-*}"
    corpus="${basename#*-}"
    mem_file="$RESULTS_DIR/${basename}.mem"

    echo "## ${scenario^} — ${corpus}"
    echo ""

    starlint_t=$(get_median "$json_file" "starlint")
    oxlint_t=$(get_median "$json_file" "oxlint")
    eslint_t=$(get_median "$json_file" "eslint")

    starlint_sd=$(get_stddev "$json_file" "starlint")
    oxlint_sd=$(get_stddev "$json_file" "oxlint")
    eslint_sd=$(get_stddev "$json_file" "eslint")

    starlint_mem=$(get_memory_kb "$mem_file" "starlint")
    oxlint_mem=$(get_memory_kb "$mem_file" "oxlint")
    eslint_mem=$(get_memory_kb "$mem_file" "eslint")

    # Detect if eslint data is present
    has_eslint=true
    if [ "$eslint_t" = "N/A" ] || [ -z "$eslint_t" ]; then
        has_eslint=false
    fi

    if [ "$has_eslint" = true ]; then
        echo "| Tool | Median | Stddev | Memory | vs eslint | vs oxlint |"
        echo "|------|--------|--------|--------|-----------|-----------|"
    else
        echo "| Tool | Median | Stddev | Memory | vs oxlint |"
        echo "|------|--------|--------|--------|-----------|"
    fi

    if [ "$has_eslint" = true ]; then
        printf "| starlint | %s | %s | %s | %s | %s |\n" \
            "$(fmt_time "$starlint_t")" \
            "$(fmt_time "$starlint_sd")" \
            "$(fmt_mem "$starlint_mem")" \
            "$(speedup "$eslint_t" "$starlint_t")" \
            "$(speedup "$oxlint_t" "$starlint_t")"

        printf "| oxlint | %s | %s | %s | %s | — |\n" \
            "$(fmt_time "$oxlint_t")" \
            "$(fmt_time "$oxlint_sd")" \
            "$(fmt_mem "$oxlint_mem")" \
            "$(speedup "$eslint_t" "$oxlint_t")"

        printf "| eslint | %s | %s | %s | — | — |\n" \
            "$(fmt_time "$eslint_t")" \
            "$(fmt_time "$eslint_sd")" \
            "$(fmt_mem "$eslint_mem")"
    else
        printf "| starlint | %s | %s | %s | %s |\n" \
            "$(fmt_time "$starlint_t")" \
            "$(fmt_time "$starlint_sd")" \
            "$(fmt_mem "$starlint_mem")" \
            "$(speedup "$oxlint_t" "$starlint_t")"

        printf "| oxlint | %s | %s | %s | — |\n" \
            "$(fmt_time "$oxlint_t")" \
            "$(fmt_time "$oxlint_sd")" \
            "$(fmt_mem "$oxlint_mem")"
    fi

    echo ""
done

echo "---"
echo "*Benchmarked with hyperfine (warmup + min runs). Memory via /usr/bin/time -v.*"
