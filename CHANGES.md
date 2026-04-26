# Changelog

## 2026-04-24

### UI

- Panes auto-scroll to the bottom on every render, so long Output / Trace /
  History / Memory contents stay pinned to the latest line instead of stranding
  the reader at the top while new activity lands off-screen

### UI

- `web/forth-bootstrap.fs` adds `2DUP`, `2DROP`, `2SWAP`, `MIN`, and `MAX`
  alongside the existing helpers, plus a `HELLO` example exercising `."`
- `web/forth-bootstrap.fs`: a Forth-source kernel-extension file with
  `NEGATE`, `ABS`, `0=`, `0<`, `NIP`, `TUCK`, `?DUP`, `*/`, `VAR`,
  `CONST`, each annotated with a stack-effect comment.
  Auto-loaded on every page init, demonstrating that the
  kernel's self-hosting primitives (`CREATE`, `DOES>`, `,`, etc.) are
  enough to define ordinary Forth vocabulary in Forth. The Source pane
  is pre-filled with this content on a fresh visit so users can read,
  edit, and re-load it; saved Source content from a previous session
  takes precedence
- `SEE` now appends ` IMMEDIATE` to its output when the target word has been
  marked via the `IMMEDIATE` primitive, so compile-time-active words are
  visible at a glance

### Language features

- `."` (string-print): in compile mode emits `OpKind::PrintStr(String)` that
  appends the captured literal to `output_line` at run time; in interpret mode
  appends directly. The token loop is now a hand-rolled char walker that
  reads whitespace-delimited tokens normally and switches to char-by-char
  mode after `."` to capture everything up to the closing `"`. Bootstrap
  gains a `: HELLO ." Hello, world!" CR ;` demo
- `( ... )` block comments: token processing now tracks `(`/`)` nesting
  depth via a new `run_tokens` helper; tokens inside block comments are
  skipped, and unmatched `)` or unclosed `(` produce diagnostic output
- `/MOD` ( a b -- rem quot ) and `*/MOD` ( a b c -- rem quot ): composite
  arithmetic primitives. `/MOD` returns both remainder and quotient in
  one operation; `*/MOD` does the canonical Forth scaled-divide
  (compute `a*b`, then divide by `c`, return both remainder and quotient)
- `\ ` (Forth-style line comment): `eval_repl` and `load_source` now
  iterate per line and break out of token processing on a `\ ` token,
  so comments in source files and REPL lines are skipped silently

### Language features

- `POSTPONE` — compile-mode helper that consumes the next token as a
  target name and emits an `OpKind::PostponeCall(name)` into the
  current word. At run time, that op appends a `CallByName(name)` to
  whatever word is currently being compiled — so an `IMMEDIATE` word
  containing `POSTPONE DUP POSTPONE +` becomes a Forth-side macro that
  splices `DUP +` into its callers. Pending tracks `pending_postpone`
  to consume the target token

- `:NONAME ... ;` — define an anonymous word and push its xt to the
  data stack. Synthesises a name like `<anon-N>` (counter on Machine),
  defines it the same way a normal `: ... ;` would, then interns the
  xt and leaves it on the stack so the caller can `EXECUTE` it,
  store it in a variable, etc.

- `CREATE` and `DOES>` — the Forth defining-word mechanism. `CREATE` is a
  primitive that captures the current `HERE` as a data-field address and
  defers the name to the next token (via a new `NextTokenConsumer::Create`
  variant). `DOES>` compiles an `OpKind::Does` op; at run time that op
  captures the remainder of the currently-executing frame into
  `Machine.pending_does` and jumps the frame past the capture. The next
  `CREATE`-consumed name picks up those ops as its `does_ops`. A new
  `Word::Created { data_addr, does_ops }` handles both the basic
  data-field-pushing behavior and the custom runtime action. Enables
  defining words in Forth itself, e.g.:

    : MY-CONST CREATE , DOES> @ ;
    7 MY-CONST SEVEN   SEVEN .    -> 7

  `SEE` renders `Word::Created` as `created @ addr N` (plus `does [ ... ]`
  if set) so the new entries stay introspectable
- `IMMEDIATE`, `[`, `]`, `LITERAL` — the gateway to user-extensible
  compiler words. `IMMEDIATE` marks `LATEST` as immediate (tracked in a
  new `immediate_words: HashSet<String>`). When an immediate-flagged
  word is seen during compilation, `dispatch_compile` executes it inline
  instead of emitting `CallByName`, so user-defined compile helpers
  work the same as built-ins. `[` pauses the current definition into
  `paused_compile: Option<Pending>` and drops out to interpret mode;
  `]` pops that back into `compiling`. `LITERAL` pops the data stack
  and emits a `PushInt` op into the in-progress body, enabling the
  classic `: FIVE [ 2 3 + ] LITERAL ;` compile-time-constant idiom
- `HERE`, `,` (comma), `LATEST` self-hosting primitives:
  `HERE` pushes the next free memory cell index (== `memory.len()`).
  `,` pops a value and appends it to memory, so `42 ,` is a
  one-cell allocator. `LATEST` pushes the xt of the most-recently
  defined word — primitives count, so a fresh machine yields the xt
  of `LATEST` itself; defining a colon word, variable, or constant
  updates it. New `define_word` helper centralises dictionary inserts
  and the `latest` field update; primitives, `;`-finalize, `VARIABLE`,
  and `CONSTANT` all flow through it

### Documentation

- README screenshot refreshed after HERE/,/LATEST, IMMEDIATE/[ ]/LITERAL,
  and SEE-immediate annotation landed; bumped cache-buster again
- README screenshot refreshed against the deployed site after the tick +
  EXECUTE and localStorage commits landed; bumped the `?ts=` cache-buster

### UI

- Source and REPL textareas persist across page reloads via
  `localStorage` (keys `sw-fth-wasm:source` and `sw-fth-wasm:repl`).
  Restored before WASM init so a saved program is visible immediately;
  saved on each `input` event. Falls back silently if `localStorage` is
  unavailable (private browsing, sandboxed iframes)

### Language features

- Tick (`'`) and `EXECUTE`: `'` consumes the next token, looks the word up in
  the dictionary, and either pushes its execution token (xt) to the data
  stack (interpret mode) or compiles a literal-push of that xt into the
  word's body (compile mode). `EXECUTE` pops an xt and runs that word.
  xts are stable indices into a new `xt_table: Vec<String>`. Same `'` token
  in either mode — the deferred `next_consumer` checks `compiling` at
  consumption time. Supports `: APPLY-DUP ' DUP EXECUTE ;` and
  `' SQUARE EXECUTE` patterns and round-trips through `SEE`
- `J` primitive: peek the outer DO loop's index (3rd from top of return stack)
- `LEAVE`: jump out of the innermost DO loop early; pops the loop's
  (limit, index) pair from the return stack so the outer flow stays balanced.
  Compile-time tracking via a per-loop list of pending leave-jumps that
  `LOOP` / `+LOOP` patch when they emit
- `+LOOP`: counted loop with a step pulled from the data stack each
  iteration. Exits when the step is non-negative and `index >= limit`, or
  when the step is negative and `index < limit`

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
