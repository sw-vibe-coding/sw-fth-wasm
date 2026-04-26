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
: 2DUP   ( a b -- a b a b )   OVER OVER ;
: 2DROP  ( a b -- )           DROP DROP ;
: 2SWAP  ( a b c d -- c d a b )  ROT >R ROT R> ;
: <=     ( a b -- flag )      > 0= ;
: >=     ( a b -- flag )      < 0= ;
: <>     ( a b -- flag )      = 0= ;
: MIN    ( a b -- min )       2DUP > IF SWAP THEN DROP ;
: MAX    ( a b -- max )       2DUP < IF SWAP THEN DROP ;
: VAR    ( -- )  CREATE 0 , ;
: CONST  ( n -- )  CREATE , DOES> @ ;

\ A small string demo using the new ." word.
: HELLO  ( -- )  ." Hello, world!" CR ;
