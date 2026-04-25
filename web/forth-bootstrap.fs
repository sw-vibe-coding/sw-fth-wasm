\ proto-forth-wasm bootstrap
\ Forth-source words layered on top of the Rust kernel. They demonstrate that
\ the kernel exposes enough self-hosting machinery (CREATE/DOES>/, LATEST/
\ IMMEDIATE/etc.) to define ordinary Forth vocabulary in Forth itself.
\
\ Inspect any of these with `SEE <name>`; list everything with `WORDS`.

: NEGATE ( n -- -n )    0 SWAP - ;
: ABS    ( n -- |n| )   DUP 0 < IF NEGATE THEN ;
: 0=     ( n -- flag )  0 = ;
: 0<     ( n -- flag )  0 < ;
: NIP    ( a b -- b )   SWAP DROP ;
: TUCK   ( a b -- b a b ) SWAP OVER ;
: ?DUP   ( n -- n n | 0 )   DUP IF DUP THEN ;
: */     ( a b c -- a*b/c ) * / ;
: VAR    ( -- )  CREATE 0 , ;
: CONST  ( n -- )  CREATE , DOES> @ ;
