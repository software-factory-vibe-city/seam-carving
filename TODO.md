# Performance Optimization Plan

## 🛠️ Implementation Steps

### 1. Memory & Allocation Optimization
- [x] **DP Buffer Reuse**: Eliminate `next_dp` allocations in the DP loop by using two pre-allocated buffers and swapping them.

### 2. Computational Efficiency
- [x] **Sobel Loop Unrolling**: Manually unroll the 3x3 window calculations to reduce overhead.
- [ ] **Wasm SIMD**: Compile Wasm with SIMD support (`-C target-feature=+simd128`) to process multiple pixels at once.
- [ ] **Pre-calculate Indices**: Avoid repeated multiplication in inner loops.

## 🧪 Validation
- [x] Run `cargo test` to ensure correctness.
- [x] Run `cargo build` to ensure compilation.
- [ ] Verify Wasm performance in the browser.
