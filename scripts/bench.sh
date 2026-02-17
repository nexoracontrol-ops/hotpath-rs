#!/usr/bin/env bash
set -euo pipefail

BENCH_CMD="cargo run -p hotpath --features=tui,hotpath,hotpath-meta,hotpath-alloc-meta --bin hotpath"

mkdir -p tmp

REF=$(git rev-parse --short HEAD)
BRANCH=$(git rev-parse --abbrev-ref HEAD)
LABEL="$BRANCH ($REF)"
OUTPUT="tmp/bench.txt"

bench_env=(
    HOTPATH_TUI_TAB=${HOTPATH_TUI_TAB:-1}
    HOTPATH_TUI_REFRESH_INTERVAL_MS=${HOTPATH_TUI_REFRESH_INTERVAL_MS:-10}
    HOTPATH_META_REPORT='functions-timing,functions-alloc,threads'
    HOTPATH_META_OUTPUT_PATH="$OUTPUT"
    HOTPATH_META_SHUTDOWN_MS=10000
    HOTPATH_META_TIMEOUT_MS=10000
    HOTPATH_META_EXCLUDE_WRAPPER=true
    RUSTFLAGS='--cfg tokio_unstable'
)

echo "==> Running: ${bench_env[*]} $BENCH_CMD"
env "${bench_env[@]}" $BENCH_CMD

tmp_header=$(mktemp)
echo "Report generated for: $LABEL" > "$tmp_header"
echo "" >> "$tmp_header"
cat "$OUTPUT" >> "$tmp_header"
mv "$tmp_header" "$OUTPUT"

reset

echo ""
echo "==> Report for: $LABEL"
echo ""
cat "$OUTPUT"
