# Seam Carving

This project implements a high-performance image resizing algorithm using seam carving. It features a Rust-based core and a WebAssembly (Wasm) implementation for browser-based resizing.

## 🤖 AI Attribution
This repository was developed completely with **Gemma 4** (specifically the [Intel/gemma-4-31B-it-int4-AutoRound](https://huggingface.co/Intel/gemma-4-31B-it-int4-AutoRound) model).

## 🌐 Browser Version (Wasm)
The browser version is provided for demonstration and accessibility. Please note:
- **Performance**: The Wasm version is slower than the native CLI implementation due to the "Marshalling Tax" (data copying between JavaScript and Wasm) and the current limitations of multi-threading in the browser.
- **Stability**: The browser implementation might be a little buggy. If you encounter rendering issues, ensure you are using a modern browser that supports Web Workers and Wasm.

## 🛠️ Getting Started
(Rest of the existing README content...)
