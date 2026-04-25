\ proto-forth-wasm bootstrap
\ Forth-source words layered on top of the Rust kernel. They demonstrate that
\ the kernel exposes enough self-hosting machinery (CREATE/DOES>/, LATEST/
\ IMMEDIATE/etc.) to define ordinary Forth vocabulary in Forth itself.
\
\ Inspect any of these with `SEE <name>`; list everything with `WORDS`.

: NEGATE 0 SWAP - ;
: ABS DUP 0 < IF NEGATE THEN ;
: 0= 0 = ;
: 0< 0 < ;
: NIP SWAP DROP ;
: TUCK SWAP OVER ;
: ?DUP DUP IF DUP THEN ;
: */ * / ;
: VAR CREATE 0 , ;
: CONST CREATE , DOES> @ ;
