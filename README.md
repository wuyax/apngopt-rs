# apngopt-rs

A fast, memory-safe, 1:1 port of the classic C++ [apngopt](https://sourceforge.net/projects/apng/files/APNG_Optimizer/) in pure Rust.

It significantly reduces the size of Animated PNG (APNG) files using a variety of compression techniques:
- **Palette Quantization:** Uses `imagequant` to convert 32-bit RGBA frames to highly optimized 8-bit palettes with alpha support.
- **Frame Optimization:** Calculates the minimal bounding box (`[x,y,w,h]`) for changed pixels between frames.
- **Alpha Overwriting:** Uses `BlendOp::Over` to replace static/identical pixels with fully transparent zeros to improve DEFLATE/Zopfli compression.
- **Multithreaded Filter Search:** Leverages `rayon` to test all 5 PNG filters (None, Sub, Up, Average, Paeth) in parallel across multiple compression backends.
- **Zopfli Compression:** Built-in high-density Zopfli engine for maximal size reduction.

## Installation

You can install it directly from `crates.io` using cargo:

```bash
cargo install apngopt-rs
```

*(Note: Currently requires a Rust toolchain to build).*

## WebAssembly (WASM) Support

`apngopt-rs` can be compiled to WebAssembly to run directly in modern web browsers or Node.js environments.

### Building for WASM

To build the WASM package, ensure you have `wasm-pack` installed:

```bash
cargo install wasm-pack
```

Then, run the build command in the project root:

```bash
# Build for Web environments (browsers, Webpack, Vite)
wasm-pack build --target web

# Build for Node.js environments
wasm-pack build --target nodejs
```

This will output a `pkg/` directory containing the `.wasm` binary and JavaScript bindings.

### Using in JavaScript/TypeScript

Once built, you can easily use the optimizer in your frontend code:

```javascript
import init, { optimize_apng_wasm } from './pkg/apngopt_rs.js';

async function run() {
    // Initialize the WASM module
    await init();
    
    // Example: Fetch an APNG file and convert to Uint8Array
    const response = await fetch('my_animation.png');
    const arrayBuffer = await response.arrayBuffer();
    const inputData = new Uint8Array(arrayBuffer);
    
    try {
        // Parameters:
        // 1. input_data: Uint8Array containing the APNG file
        // 2. z_method: 0=zlib, 1=7zip(zlib), 2=zopfli
        // 3. iterations: zopfli iterations (default: 15)
        // 4. disable_imagequant: 0=false(enabled), 1=true(disabled)
        const optimizedData = optimize_apng_wasm(inputData, 0, 15, 0);
        
        console.log("Original size:", inputData.length);
        console.log("Optimized size:", optimizedData.length);
        
        // Create a blob and URL to display or download the optimized image
        const blob = new Blob([optimizedData], { type: 'image/png' });
        const url = URL.createObjectURL(blob);
        // ... use the URL ...
    } catch (e) {
        console.error("Optimization failed:", e);
    }
}

run();
```

*(Note: When compiled to WASM, multithreading via `rayon` is disabled, and execution happens synchronously on the single JS thread. For heavy processing with Zopfli, consider using Web Workers to prevent blocking the UI).*

## Usage

```bash
apngopt-rs [options] anim.png [anim_opt.png]
```

### Options

*   `-z0` : zlib compression (default)
*   `-z1` : 7zip compression (currently maps to best zlib in Rust)
*   `-z2` : zopfli compression (Extremely slow but provides the smallest file size)
*   `-i##` : number of zopfli iterations (default: 15)
*   `-d##` : disable imagequant palette compression (0 = false/enabled, 1 = true/disabled)

**Examples:**

1. Default optimization (Imagequant + Zlib):
```bash
apngopt-rs my_animation.png
```

2. Maximum compression (Imagequant + Zopfli with 30 iterations):
```bash
apngopt-rs -z 2 -i 30 my_animation.png
```

3. True-color lossless mode (Disable imagequant, preserve all original RGBA values):
```bash
apngopt-rs -d 1 -z 2 my_animation.png
```

## Why a Rust Port?

The original C++ `apngopt` is a fantastic tool, but its monolithic design makes it difficult to embed into modern backends (like web servers or data pipelines) safely. It heavily relies on global states, pointer arithmetic, and single-threaded execution.

This Rust version:
1. **Thread-Safe**: Uses zero global states. You can run `apngopt` on hundreds of images concurrently in the same process.
2. **Multi-threaded**: Tests PNG filters and compression strategies using the `rayon` threadpool, making it significantly faster on multi-core machines than the original.
3. **Memory Safe**: No segfaults or buffer overflows when encountering malformed APNG files.
4. **Embeddable**: Designed as a library, its core logic (`decoder`, `optimizer`, `quantize`, `compress`, `encoder`) can easily be imported into other Rust projects.

## Architecture

*   **Decoder**: Driven by the `png` crate. Flattens out all `dispose_op` and `blend_op` instructions to recreate full `RGBA` canvases in memory.
*   **Quantizer**: Uses the `imagequant` crate to generate a global 256-color palette optimal for all frames.
*   **Optimizer**: Compares adjacent frames to calculate a minimum dirty rect bounding box, and tests if `BlendOp::Over` can be used to zero-out redundant pixels.
*   **Compressor**: Runs parallel threads applying different PNG filters and deflating via `flate2` or `zopfli`.
*   **Encoder**: Re-assembles the optimized chunks (`IHDR`, `PLTE`, `tRNS`, `acTL`, `fcTL`, `IDAT`, `fdAT`, `IEND`), recalculates CRC32s, and writes to disk.

## License

This project is licensed under the same [zlib License](https://opensource.org/licenses/Zlib) as the original APNG Optimizer.
