#!/bin/bash
# Ralph Wiggum loop for starlint
# Usage: ./loop.sh [max_iterations]  (default: 10)

MAX_ITERATIONS=${1:-10}
ITERATION=0
CURRENT_BRANCH=$(git branch --show-current)

[ ! -f "PROMPT_build.md" ] && echo "Error: PROMPT_build.md not found" && exit 1

while [ $ITERATION -lt $MAX_ITERATIONS ]; do
    ITERATION=$((ITERATION + 1))
    echo -e "\n======== RALPH ITERATION $ITERATION / $MAX_ITERATIONS ========\n"

    # Use 'happy' instead of 'claude' so session streams to phone
    cat PROMPT_build.md | happy -p auto \
        --claude-arg "--dangerously-skip-permissions" \
        --claude-arg "--output-format=stream-json" \
        --claude-arg "--model" --claude-arg "opus" \
        --claude-arg "--verbose"

    EXIT_CODE=$?

    # Push after each iteration
    git push origin "$CURRENT_BRANCH" 2>/dev/null || git push -u origin "$CURRENT_BRANCH"

    # Check if beads has no more ready work
    READY=$(bd ready --json 2>/dev/null | jq 'length')
    if [ "$READY" = "0" ]; then
        echo "All beads tasks complete!"
        happy notify -p "All tasks complete!" -t "starlint"
        break
    fi
done

echo "Ralph loop finished after $ITERATION iterations."
happy notify -p "Ralph loop done ($ITERATION iterations)" -t "starlint"
