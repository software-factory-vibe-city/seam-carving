# Seam Stitching

Seam Stitching is a Rust implementation of the **Seam Carving** algorithm, used for content-aware image resizing. Unlike standard scaling, seam carving removes pixels along paths (seams) of low energy (low visual importance), allowing the image to be resized while preserving important features.

## Features

- **Content-Aware Resizing**: Reduces width and height by removing low-energy seams.
- **Vertical and Horizontal Seams**: Supports resizing in both dimensions.
- **Command Line Interface**: Easy-to-use CLI for processing images.

## Installation

Ensure you have Rust and Cargo installed. Then, clone the repository and build the project:

```bash
git clone <repository-url>
cd seam-stitching
cargo build --release
```

## Usage

You can use the compiled binary to resize images. Run the following command:

```bash
cargo run -- --input input.jpg --output output.jpg --width 800 --height 600
```

### Arguments

- `-i, --input <path>`: Path to the input image.
- `-o, --output <path>`: Path where the resized image will be saved.
- `-w, --width <u32>`: Target width for the image (optional).
- `-H, --height <u32>`: Target height for the image (optional).

## How it Works

1. **Energy Map**: The algorithm calculates an energy map of the image based on the gradient of pixel intensity. High-energy areas (edges, textures) are preserved, while low-energy areas (flat backgrounds) are candidates for removal.
2. **Seam Identification**: Using dynamic programming, the algorithm finds the path (seam) from top to bottom (or left to right) with the lowest cumulative energy.
3. **Seam Removal**: The identified seam is removed from the image, and the process repeats until the target dimensions are achieved.

## Dependencies

- `image`: Used for image processing and manipulation.
- `clap`: Used for command-line argument parsing.
