# proto-forth-wasm

Minimal plain HTML + JS + Rust/WASM prototype for a proto-Forth workbench.

**Live demo:** <https://sw-vibe-coding.github.io/sw-fth-wasm/>

![screenshot](images/screenshot.png?ts=1777223675862)

## What it does

- REPL + Source textareas; live-updated Stack / Dictionary / Output / History / Trace panes
- Rust/WASM `Machine` owns the data stack, return stack, memory, dictionary, tokenizer, compiler, and VM loop
- Colon definitions compile to an opcode IR; user-word execution runs on an iterative VM loop (no host-stack recursion for nested calls)
- Vocabulary covers arithmetic, stack shuffling, comparisons, conditionals (`IF/ELSE/THEN`), post- and pre-test loops (`BEGIN/UNTIL`, `BEGIN/WHILE/REPEAT`), counted loops (`DO/LOOP` with `I`), memory (`VARIABLE`, `CONSTANT`, `@`, `!`, `+!`, `ALLOT`), return stack (`>R R> R@`), I/O (`CR EMIT SPACE`), and introspection (`SEE <word>`, `WORDS`). See [CHANGES.md](CHANGES.md) for the full feature timeline.

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
- dictionary mapping word names to primitive implementations, user-word op bodies, variables, and constants

The same WASM module can grow more Forth-owned behavior (self-hosting-oriented compiler features, a memory pane, tick + `EXECUTE`, etc.) without changing the browser shell very much.

## Related Repositories

- [sw-cor24-forth](https://github.com/sw-embed/sw-cor24-forth) — sibling DTC Forth targeting the COR24 24-bit RISC ISA
- [sw-cor24-project](https://github.com/sw-embed/sw-cor24-project) — COR24 ecosystem hub

## Links

- Blog: [Software Wrighter Lab](https://software-wrighter-lab.github.io/)
- Discord: [Join the community](https://discord.com/invite/Ctzk5uHggZ)
- YouTube: [Software Wrighter](https://www.youtube.com/@SoftwareWrighter)

## Copyright

Copyright (c) 2026 Michael A. Wright

## License

MIT License. See [LICENSE](LICENSE) for the full text.
