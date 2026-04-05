# Performance Optimization Plan

## 🚀 High-Level Goals
Transform the library into a high-performance implementation by minimizing memory allocations, reducing redundant calculations, and unifying symmetric logic.

## 🛠️ Implementation Steps

### 1. Architectural Refactoring (DRY)
- [ ] Unify `resize_width` and `resize_height` into a single core resizing engine.
- [ ] Implement a rotation-based approach: to resize height, rotate image 90°, resize width, and rotate back. This removes 50% of the duplicated logic.

### 2. Memory & Allocation Optimization
- [ ] **Eliminate Image Re-allocations**: Replace `remove_vertical_seam` (which creates a new image) with an in-place pixel shifting mechanism.
- [ ] **Optimized DP Tables**: Reduce DP memory from $O(W \times H)$ to $O(W)$ by using a sliding window of two rows for energy accumulation.
- [ ] **Zero-Copy Conversions**: Minimize conversions between `DynamicImage`, `RgbImage`, and `GrayImage`.

### 3. Computational Efficiency
- [ ] **Sobel Energy Operator**: Replace the simple absolute difference with a Sobel operator for higher quality and better-defined seams.
- [ ] **SIMD/Parallelism**: Evaluate the use of `rayon` for parallel energy calculation (since each pixel's energy is independent).
- [ ] **Pre-calculate Indices**: Avoid repeated multiplication in inner loops by using pointer arithmetic or pre-calculated offsets.

### 4. API & Ergonomics
- [ ] **Logging Integration**: Replace `println!` with the `log` crate.
- [ ] **Progress Tracking**: Implement `indicatif` for professional progress bars during long resize operations.
- [ ] **Result-based API**: Ensure all functions use a custom `Error` type for better debugging.

## 🧪 Validation
- [ ] Update test suite to ensure refactored code maintains correctness.
- [ ] Add benchmarks using `criterion` to quantify performance gains.
