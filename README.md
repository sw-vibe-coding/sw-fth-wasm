# proto-forth-wasm

Minimal plain HTML + JS + Rust/WASM prototype for a proto-Forth workbench.

## What it does

- REPL input from a textarea
- Source input from a second textarea
- Stack / Dictionary / Output / History / Trace panes
- Rust/WASM `Machine` owns the stack, output, history, dictionary, trace, tokenization, and primitive dispatch
- Supported tokens: integer literals, `DUP`, `+`, `*`, `.`, `.S`, `CLEAR`, `:` … `;` (user word definitions)

## Build

Requirements:

- Rust toolchain with `rustup`
- `wasm-bindgen-cli`

Install `wasm-bindgen-cli` if needed:

```bash
cargo install wasm-bindgen-cli
```

Build:

```bash
./scripts/build.sh
```

Serve the `web/` directory:

```bash
cd web
python3 -m http.server 8000
```

Then open:

```text
http://localhost:8000
```

## Why this shape matters

This is the first clean split between:

- browser shell (HTML + tiny JS)
- runtime engine (`Machine` in Rust/WASM)
- dictionary mapping word names to primitive implementations

Later, the same WASM module can grow colon definitions and more Forth-owned behavior without changing the browser shell very much.
