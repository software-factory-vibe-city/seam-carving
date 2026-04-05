# Seam Carving Optimization Roadmap

## Goal
Eliminate "squishing" and structural distortion in image resizing by moving from backward energy (local gradients) to **Forward Energy** to achieve correct seam carving.

## Current Status
- [x] Basic Seam Carving implementation (Sobel-based).
- [x] Basic Seam Path Smoothing.
- [ ] **Forward Energy implementation** (Crucial for fixing "squishing").

## Priority Roadmap

### 1. Forward Energy Implementation (High Priority)
The current backward energy (Sobel) only looks at the current state, leading to seams that cut through smooth objects (like balloons). Forward energy calculates the energy cost based on the *new* edges created after the seam is removed.
- [ ] Implement `calculate_forward_energy` function.
- [ ] Update the main loop to recalculate energy based on the forward energy model.
- [ ] Validate that seams now "flow" around structural objects rather than through them.

### 2. Structural Preservation Refinement (Medium Priority)
- [ ] Fine-tune the DP path selection to prevent systematic bias (e.g., left-side squishing).
- [ ] Evaluate and refine energy weighting to ensure a perfect balance between background removal and object preservation.

## Validation Plan
- [ ] **Test Case**: `images/in/baloons.png` $\rightarrow$ 50% width.
- [ ] **Success Criteria**: 
    - Background (blue) is removed.
    - Balloons retain their circular shape (no squishing).
    - No systematic shift of the image content to one side.
- [ ] **Technical Validation**: `cargo test` and `cargo build --release`.
