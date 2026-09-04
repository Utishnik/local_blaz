#!/bin/bash
set -e

BENCH_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$BENCH_DIR"

MODE="${1:-encode}"
ITERATIONS="${2:-20000}"
DATA="${3:-The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs.}"
LARGE="${4:-}"

echo "=== Hotpath profile: bench_rust_hotpath ==="
echo ""
echo "[1/1] Building bench_rust_hotpath..."
cargo build --release --bin bench_rust_hotpath 2>&1 | tail -1
echo "  -> OK"
echo ""

if [ -n "$LARGE" ]; then
    echo "Running: --large $LARGE, mode=$MODE, iters=$ITERATIONS"
    ./target/release/bench_rust_hotpath --large "$LARGE" SuperSecret123 "$MODE" "$ITERATIONS" 2>&1
else
    echo "Running: mode=$MODE, iters=$ITERATIONS"
    ./target/release/bench_rust_hotpath "$MODE" SuperSecret123 "$DATA" "$ITERATIONS" 2>&1
fi