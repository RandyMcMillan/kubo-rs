# wasm-hybrid-wasi

Demonstrates compiling the native Rust `nostr` crate (which depends on the C `secp256k1` library) to `wasm32-wasip1` and running it via a WASI runtime.

## Why WASI?

The standard browser target `wasm32-unknown-unknown` has no OS interface, so C libraries like `secp256k1-sys` cannot compile. `wasm32-wasip1` provides a POSIX-like system interface (WASI) that allows `clang` to compile C code for WebAssembly.

## Trade-offs

| Aspect | `wasm32-unknown-unknown` | `wasm32-wasip1` |
|---|---|---|
| C libraries | ❌ No | ✅ Yes (with wasi-sdk) |
| Browser runner | Trunk, wasm-pack | WASI shim required |
| Bundle size | Small | Larger (WASI polyfill) |
| Tooling maturity | Mature | Emerging |

## Prerequisites

```bash
rustup target add wasm32-wasip1
```

You also need **wasi-sdk** for C compilation:

- **macOS**: Download the latest release from [WebAssembly/wasi-sdk](https://github.com/WebAssembly/wasi-sdk/releases) and extract to `/opt/wasi-sdk` or set `WASI_SDK_PATH`.
- **Linux**: `apt install wasi-sdk` on Debian/Ubuntu, or download the release tarball.
- **Windows**: Download the release ZIP and set `WASI_SDK_PATH`.

```bash
export WASI_SDK_PATH=/opt/wasi-sdk
```

## Build

```bash
cd examples/wasm-hybrid-wasi
cargo build --target wasm32-wasip1 --release
```

## Run (CLI)

With [Wasmtime](https://wasmtime.dev):

```bash
wasmtime target/wasm32-wasip1/release/wasm-hybrid-wasi.wasm
```

With [Wasmer](https://wasmer.io):

```bash
wasmer target/wasm32-wasip1/release/wasm-hybrid-wasi.wasm
```

## Run (Browser)

Open `index.html` in a browser after building. The page includes a WASI shim (`@bjorn3/browser_wasi_shim`) that polyfills the required system calls in the browser.

## Comparison

See also `examples/wasm-hybrid/` which uses the same functionality but bridges to a JavaScript `nostr-tools` library via `wasm-bindgen` instead of compiling C code via WASI.
