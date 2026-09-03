#!/bin/bash
set -e

BENCH_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$BENCH_DIR"

echo "=== obxrac32b64 Benchmark: Rust vs C++ ==="
echo ""

# Clang для сборки (если есть clang++-22/21 -> используем, иначе системный clang++)
if command -v clang++-22 &> /dev/null; then
    CXX=clang++-22
elif command -v clang++-21 &> /dev/null; then
    CXX=clang++-21
else
    CXX=clang++
fi
echo "[1/2] Building C++ ($($CXX --version | head -1))..."

# Максимальные флаги оптимизации + целевой процессор текущей машины
CPPFLAGS="-O3 -march=native -mtune=native -flto=full \
    -ffast-math -funroll-loops \
    -fomit-frame-pointer \
    -fvectorize -fslp-vectorize \
    -fno-exceptions -fno-rtti \
    -fstrict-aliasing -fstrict-overflow \
    -finline-functions -fno-semantic-interposition \
    -fvisibility=hidden \
    -falign-functions=64 -falign-loops=64"

# Использовать lld если он доступен
if command -v ld.lld &> /dev/null; then
    CPPFLAGS="$CPPFLAGS -fuse-ld=lld"
fi

$CXX $CPPFLAGS -o bench_cpp bench_cpp.cpp -std=c++20
$CXX $CPPFLAGS -o test_cpp test_cpp.cpp -std=c++20
echo "  -> OK"

# --- Build Rust ---
echo "[2/2] Building Rust (release: panic=abort, LTO, codegen-units=1, native)..."
cargo build --release 2>&1 | tail -1
echo "  -> OK"

echo ""
echo "--- Build done ---"
echo ""

RUST_BENCH=target/release/bench_rust
RUST_BENCH_SIMD=target/release/bench_rust_simd
RUST_TEST=target/release/test_rust

KEY="SuperSecret123"
TEXT_SHORT="Hello World"
TEXT_MED="The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs."
TEXT_LONG=$(python3 -c "print('A' * 1000)")

ITERATIONS=50000
WARMUP_ITERS=2000

run_bench() {
    local label="$1"
    local mode="$2"
    local key="$3"
    local text="$4"
    local iters="$5"

    echo "=== $label (mode=$mode, text_len=${#text}, iters=$iters) ==="

    echo "  [warmup] C++ x${WARMUP_ITERS}..."
    ./bench_cpp "$mode" "$key" "$text" "$WARMUP_ITERS" > /dev/null
    echo -n "  C++   : "
    ./bench_cpp "$mode" "$key" "$text" "$iters"

    echo "  [warmup] Rust x${WARMUP_ITERS}..."
    "$RUST_BENCH" "$mode" "$key" "$text" "$WARMUP_ITERS" > /dev/null
    echo -n "  Rust   : "
    "$RUST_BENCH" "$mode" "$key" "$text" "$iters"

    echo "  [warmup] Rust-SIMD x${WARMUP_ITERS}..."
    "$RUST_BENCH_SIMD" "$mode" "$key" "$text" "$WARMUP_ITERS" > /dev/null
    echo -n "  RustSIMD: "
    "$RUST_BENCH_SIMD" "$mode" "$key" "$text" "$iters"
    echo ""
}

echo "=============================="
echo "  ENCODE BENCHMARKS"
echo "=============================="
run_bench "Short text" encode "$KEY" "$TEXT_SHORT" $ITERATIONS
run_bench "Medium text" encode "$KEY" "$TEXT_MED" $ITERATIONS
run_bench "Long text (1000 chars)" encode "$KEY" "$TEXT_LONG" 10000

ENCODED_SHORT=$(./test_cpp encode "$KEY" "$TEXT_SHORT")
ENCODED_MED=$(./test_cpp encode "$KEY" "$TEXT_MED")
ENCODED_LONG=$(./test_cpp encode "$KEY" "$TEXT_LONG")

echo "=============================="
echo "  DECODE BENCHMARKS"
echo "=============================="
run_bench "Short text" decode "$KEY" "$ENCODED_SHORT" $ITERATIONS
run_bench "Medium text" decode "$KEY" "$ENCODED_MED" $ITERATIONS
run_bench "Long text (1000 chars)" decode "$KEY" "$ENCODED_LONG" 10000

echo "=============================="
echo "  CORRECTNESS CHECK"
echo "=============================="

CPP_ENC=$("./test_cpp" encode "$KEY" "$TEXT_SHORT")
RUST_ENC=$("$RUST_TEST" encode "$KEY" "$TEXT_SHORT")
SIMD_ENC=$("$RUST_BENCH_SIMD" encode "$KEY" "$TEXT_SHORT" 1 | sed -E 's/.*avg=[0-9.]+us\/iter \|.*//')
CPP_DEC=$("$RUST_TEST" decode "$KEY" "$CPP_ENC")
RUST_DEC=$("$RUST_TEST" decode "$KEY" "$ENCODED_SHORT")

echo "  Input:    $TEXT_SHORT"
echo "  Key:      $KEY"
echo ""
echo "  C++  enc: $CPP_ENC"
echo "  Rust enc: $RUST_ENC"
echo ""
echo "  C++  dec: $CPP_DEC"
echo "  Rust dec: $RUST_DEC"
echo ""

if [ "$CPP_ENC" = "$RUST_ENC" ]; then
    echo "  [OK] Encode outputs MATCH"
else
    echo "  [MISMATCH] Encode outputs DIFFER"
fi

if [ "$TEXT_SHORT" = "$CPP_DEC" ] && [ "$TEXT_SHORT" = "$RUST_DEC" ]; then
    echo "  [OK] Decode roundtrip correct"
else
    echo "  [FAIL] Decode roundtrip failed"
fi

echo ""
echo "=== Benchmark Complete ==="
