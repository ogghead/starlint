#!/usr/bin/env bash
# Update the Benchmarks section of README.md from benchmark results.
#
# Reads JSON/mem files from benches/results/ and replaces the content
# between <!-- BENCHMARKS_START --> and <!-- BENCHMARKS_END --> markers.
#
# Usage:
#   ./benches/update-readme.sh            # Update README.md in-place
#   ./benches/update-readme.sh --dry-run  # Print the generated section without modifying README
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RESULTS_DIR="$SCRIPT_DIR/results"
README="$REPO_ROOT/README.md"

DRY_RUN=false
if [[ "${1:-}" == "--dry-run" ]]; then
    DRY_RUN=true
fi

# ── Validation ────────────────────────────────────────────────────────────

if [ ! -d "$RESULTS_DIR" ] || ! ls "$RESULTS_DIR"/equivalent-*.json &>/dev/null; then
    echo "ERROR: No benchmark results found. Run ./benches/run.sh first." >&2
    exit 1
fi

if [ ! -f "$README" ]; then
    echo "ERROR: README.md not found at $README" >&2
    exit 1
fi

if ! grep -q '<!-- BENCHMARKS_START -->' "$README"; then
    echo "ERROR: Missing <!-- BENCHMARKS_START --> marker in README.md" >&2
    exit 1
fi

# ── Helpers (shared with report.sh) ───────────────────────────────────────

get_median() {
    local json_file="$1" tool_name="$2"
    jq -r ".results[] | select(.command | contains(\"$tool_name\")) | .median" "$json_file" 2>/dev/null || echo ""
}

fmt_time() {
    local val="$1"
    if [ -z "$val" ] || [ "$val" = "N/A" ]; then
        echo "N/A"
        return
    fi
    awk "BEGIN { v=$val; if (v < 1) printf \"%.0fms\", v*1000; else printf \"%.2fs\", v }"
}

get_memory_kb() {
    local mem_file="$1" tool_name="$2"
    if [ -f "$mem_file" ]; then
        grep "^$tool_name " "$mem_file" 2>/dev/null | awk '{print $2}' || echo ""
    fi
}

fmt_mem() {
    local kb="$1"
    if [ -z "$kb" ]; then
        echo ""
        return
    fi
    awk "BEGIN { mb=$kb/1024; if (mb >= 1024) printf \"%.1f GB\", mb/1024; else printf \"%.0f MB\", mb }"
}

# Format a cell: "time (mem)" — bold if is_fastest is true
fmt_cell() {
    local time_s="$1" mem_kb="$2" is_fastest="$3"
    local t m
    t="$(fmt_time "$time_s")"
    m="$(fmt_mem "$mem_kb")"
    local cell
    if [ -n "$m" ]; then
        cell="$t ($m)"
    else
        cell="$t"
    fi
    if [ "$is_fastest" = "true" ]; then
        echo "**$cell**"
    else
        echo "$cell"
    fi
}

# Determine which tool is fastest
fastest_tool() {
    local s="$1" o="$2" e="$3"
    if [ -z "$s" ] || [ -z "$o" ] || [ -z "$e" ]; then
        echo "starlint"
        return
    fi
    awk "BEGIN {
        s=$s; o=$o; e=$e
        if (s <= o && s <= e) print \"starlint\"
        else if (o <= s && o <= e) print \"oxlint\"
        else print \"eslint\"
    }"
}

# Get file count for a corpus
get_file_count() {
    local corpus="$1"
    local counts_file="$RESULTS_DIR/file-counts.txt"
    if [ -f "$counts_file" ]; then
        grep "^$corpus " "$counts_file" 2>/dev/null | awk '{print $2}' || echo "?"
    else
        echo "?"
    fi
}

# ── Generate markdown ─────────────────────────────────────────────────────

CORPORA=(express date-fns grafana)

generate_table() {
    local scenario="$1"
    local output=""

    output+="| Corpus | Files | starlint | oxlint | eslint |"$'\n'
    output+="|--------|------:|----------|--------|--------|"$'\n'

    for corpus in "${CORPORA[@]}"; do
        local json_file="$RESULTS_DIR/${scenario}-${corpus}.json"
        local mem_file="$RESULTS_DIR/${scenario}-${corpus}.mem"

        if [ ! -f "$json_file" ]; then
            continue
        fi

        local nfiles
        nfiles="$(get_file_count "$corpus")"

        local s_t o_t e_t s_m o_m e_m
        s_t="$(get_median "$json_file" "starlint")"
        o_t="$(get_median "$json_file" "oxlint")"
        e_t="$(get_median "$json_file" "eslint")"
        s_m="$(get_memory_kb "$mem_file" "starlint")"
        o_m="$(get_memory_kb "$mem_file" "oxlint")"
        e_m="$(get_memory_kb "$mem_file" "eslint")"

        local winner
        winner="$(fastest_tool "${s_t:-999}" "${o_t:-999}" "${e_t:-999}")"

        local s_cell o_cell e_cell
        s_cell="$(fmt_cell "$s_t" "$s_m" "$([ "$winner" = "starlint" ] && echo true || echo false)")"
        o_cell="$(fmt_cell "$o_t" "$o_m" "$([ "$winner" = "oxlint" ] && echo true || echo false)")"
        e_cell="$(fmt_cell "$e_t" "$e_m" "$([ "$winner" = "eslint" ] && echo true || echo false)")"

        # Format file count with commas
        local nfiles_fmt
        nfiles_fmt="$(printf "%'d" "$nfiles" 2>/dev/null || echo "$nfiles")"

        output+="| ${corpus} | ${nfiles_fmt} | ${s_cell} | ${o_cell} | ${e_cell} |"$'\n'
    done

    echo "$output"
}

SECTION=""
SECTION+="Compared against [oxlint](https://oxc.rs) and [eslint](https://eslint.org) on real-world codebases with 20 equivalent lint rules."$'\n'
SECTION+=""$'\n'
SECTION+="$(generate_table "equivalent")"
SECTION+=""$'\n'

# Add full defaults in a collapsible section if results exist
if ls "$RESULTS_DIR"/default-*.json &>/dev/null 2>&1; then
    SECTION+="<details>"$'\n'
    SECTION+="<summary>Full defaults (all rules enabled per tool)</summary>"$'\n'
    SECTION+=""$'\n'
    SECTION+="$(generate_table "default")"$'\n'
    SECTION+="</details>"$'\n'
    SECTION+=""$'\n'
fi

SECTION+="*Last updated: $(date -u '+%Y-%m-%d'). Benchmarked with [hyperfine](https://github.com/sharkdp/hyperfine) (3 warmup, 10+ runs).*"

# ── Output / Replace ─────────────────────────────────────────────────────

if [ "$DRY_RUN" = "true" ]; then
    echo "$SECTION"
    exit 0
fi

# Replace content between markers in README.md
# Use awk to handle multi-line replacement cleanly
SECTION_ESCAPED="$SECTION"
awk -v section="$SECTION_ESCAPED" '
    /<!-- BENCHMARKS_START -->/ {
        print
        print section
        skip = 1
        next
    }
    /<!-- BENCHMARKS_END -->/ {
        skip = 0
    }
    !skip { print }
' "$README" > "$README.tmp"

mv "$README.tmp" "$README"
echo "README.md updated with benchmark results."
