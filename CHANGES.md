# Changelog

## 2026-04-24

### Documentation

- README screenshot refreshed against the live site after the Memory pane and
  Output cleanup landed; bumped the `?ts=` cache-buster

### UI

- Memory pane: new `#memoryPane` textarea and `Machine::get_memory_text()`
  that prints one line per cell as `[addr] value`, so `VARIABLE`, `ALLOT`,
  `!`, `+!` are visible alongside the Stack and Dictionary
- Output pane is now user-facing only. Per-op chatter (literal pushes, stack
  ops, arithmetic results, return-stack moves, memory fetch/store, user-word
  call banners, DO-loop entry messages) moved out of Output — the Trace pane
  remains the debug log. Errors, `.`, `.S`, `WORDS`, `SEE`, compile
  confirmations (`defined X`, `VARIABLE X at addr N`, `CONSTANT X = V`),
  reset banner, and startup banners stay
- `.` is now Forth-style: it appends `<n> ` to the in-progress `output_line`
  instead of emitting a new line, so `5 . 6 . 7 .` reads as `5 6 7 ` on a
  single line. `eval_repl` / `load_source` flush any partial `output_line`
  when the input finishes, so results always appear before the next prompt

### Documentation

- README: add live-demo link (<https://sw-vibe-coding.github.io/sw-fth-wasm/>),
  inline screenshot at `images/screenshot.png` (cache-busted via `?ts=`
  query), up-to-date vocabulary summary, and a standard epilog (Related
  Repositories, Links, Copyright, License) matching the rest of the
  Software Wrighter ecosystem

### Deployment

- GitHub Pages workflow (`.github/workflows/pages.yml`): on push to `main`,
  installs the `wasm32-unknown-unknown` target, caches cargo + `target/`,
  installs `wasm-bindgen-cli`, runs `scripts/build.sh`, substitutes build-info
  placeholders in `web/index.html`, writes `web/.nojekyll`, and deploys `web/`
  via `actions/deploy-pages@v4`
- Page footer: MIT license, copyright, Blog / Discord / YouTube / Changes
  links, and a build-info span (host · short SHA · UTC timestamp) injected at
  deploy time; `main.js` falls back to `dev` when the placeholders are
  unsubstituted (local preview)
- GitHub corner SVG links to the repo from the top-right of every page
- Add `LICENSE` (MIT) and `CHANGES.md` (this file)

### Language features

- Counted loops: `DO … LOOP` with `I` primitive (current index); `ALLOT` for
  growing memory cells, enabling `VARIABLE X N ALLOT` array patterns
- Pre-test loop form: `BEGIN … WHILE … REPEAT`
- Introspection & I/O: `WORDS` primitive; `CR`, `EMIT`, `SPACE` with an
  `output_line` buffer so `EMIT`/`SPACE` accumulate inline and `CR` flushes
- Memory model: `memory: Vec<Value>` cell array with bounds-checked addressing;
  `VARIABLE <name>` and `<val> CONSTANT <name>` define words that push an
  address or value; `@`, `!`, `+!` read/write/add-to cells
- Decompile: `SEE <word>` prints source-like form for user words and a one-line
  description for primitives, variables, constants; `THEN` and `BEGIN` emit
  `Noop` marker ops so the decompile round-trips structural keywords
- Return stack: `>R`, `R>`, `R@`
- Post-test loop form: `BEGIN … UNTIL`; arithmetic `-`, `/`, `MOD` with
  divide-by-zero guards
- Conditionals: `IF … ELSE … THEN` (nesting supported), Forth-style flags
  (`-1` true, `0` false) via `=`, `<`, `>`
- Stack shuffling: `SWAP`, `DROP`, `OVER`, `ROT`

### Compiler

- `Pending.cf_stack` tracks open `IF`/`ELSE`/`BEGIN`/`WHILE`/`DO` positions;
  `;` rejects and discards a definition if the stack is non-empty
- `next_consumer: Option<NextTokenConsumer>` unifies the "next token names
  something" state shared by `SEE`, `VARIABLE`, and `CONSTANT`

### Build

- `scripts/build.sh` resolves `ROOT_DIR` relative to the project root so it
  runs cleanly from any working directory
- README updated to point at `./scripts/build.sh` and cover the new vocabulary

## 2026-04-23

### Runtime

- Step 9 VM loop: recursive user-word execution replaced by an iterative loop
  over `Vec<Frame>`. Each frame carries `(ops, pc, return_label)`; nested user
  calls push frames instead of host-stack recursing, and the caller's trace
  line is emitted on frame pop to preserve post-expansion ordering

### Initial implementation

- Rust/WASM `Machine` owns the data stack, output log, history, trace, and
  dictionary; exposed via `wasm_bindgen`
- Browser shell (plain HTML + tiny JS) with REPL textarea, Source textarea,
  and live-updated Stack / Dictionary / Output / History / Trace panes
- Colon definitions (`: NAME … ;`) compile to `Word::User(Vec<Op>)`; opcode IR
  is `PushInt`, `CallPrim`, `CallByName` — literals and primitives resolve at
  compile time, user-to-user calls stay late-bound
- Initial primitives: `DUP`, `+`, `*`, `.`, `.S`, `CLEAR`
- Hits the research.txt §12 milestone in-browser:
  `: SQUARE DUP * ; 5 SQUARE .` → `25`
