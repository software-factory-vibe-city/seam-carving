# Performance Optimization & Wasm Migration Plan

## 🚀 High-Level Goals
Transform the library into a high-performance implementation and adapt it for WebAssembly (Wasm) to enable browser-based image resizing.

## 🛠️ Implementation Steps

### 1. Architectural Refactoring (DRY)
- [x] Unify `resize_width` and `resize_height` into a single core resizing engine.
- [x] Implement a rotation-based approach: to resize height, rotate image 90°, resize width, and rotate back.

### 2. Memory & Allocation Optimization
- [x] **Eliminate Image Re-allocations**: Replace `remove_vertical_seam` with an in-place pixel shifting mechanism.
- [x] **Optimized DP Tables**: Reduce DP memory from $O(W \times H)$ to $O(W)$ for calculation.
- [x] **Zero-Copy Conversions**: Minimize conversions between `DynamicImage`, `RgbImage`, and `GrayImage`.

### 3. Computational Efficiency
- [x] **Sobel Energy Operator**: Replace simple difference with Sobel operator for higher quality.
- [ ] **SIMD/Parallelism**: Evaluate the use of `rayon` for parallel energy calculation.
- [ ] **Pre-calculate Indices**: Avoid repeated multiplication in inner loops.

### 4. API & Ergonomics
- [ ] **Logging Integration**: Replace `println!` with the `log` crate.
- [ ] **Progress Tracking**: Implement `indicatif` for professional progress bars.
- [ ] **Result-based API**: Ensure all functions use a custom `Error` type.

### 5. WebAssembly (Wasm) Migration
- [ ] **Tooling Setup**: Install `wasm-pack` and configure the `wasm32-unknown-unknown` target.
- [ ] **Wasm Bindings**: Add `wasm-bindgen` and create a JS-accessible wrapper for the resizing functions.
- [ ] **Frontend Implementation**: Create a simple HTML/JS interface to upload images, call the Wasm module, and display results on a `<canvas>`.
- [ ] **Memory Bridge**: Optimize data transfer between JS and Wasm to avoid unnecessary copies.

## 🧪 Validation
- [x] Update test suite to ensure refactored code maintains correctness.
- [x] Add benchmarks using `criterion` to quantify performance gains.
- [ ] Verify Wasm functionality in a modern browser (Chrome/Firefox).
