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
: WITHIN ( n lo hi -- flag )  >R OVER <= SWAP R> < AND ;
: MIN    ( a b -- min )       2DUP > IF SWAP THEN DROP ;
: MAX    ( a b -- max )       2DUP < IF SWAP THEN DROP ;
: VAR    ( -- )  CREATE 0 , ;
: CONST  ( n -- )  CREATE , DOES> @ ;

\ Compile a call to the word currently being defined. Used inside : ... ;
\ for self-recursive references that don't depend on dict insertion order.
: RECURSE  ( -- )  LATEST COMPILE, ; IMMEDIATE

\ Number-base helpers: BASE is a kernel-resident variable at memory[0].
\ . / .S / literal parsing all consult it.
: HEX      ( -- )  16 BASE ! ;
: DECIMAL  ( -- )  10 BASE ! ;

\ Forth-style switch/case. Built entirely from POSTPONE + IF/ELSE/THEN +
\ BEGIN/?DUP/WHILE/REPEAT — no Rust changes were needed for this.
\ CASE pushes a 0 marker on the compile-time data stack; each ENDOF
\ increments the count; ENDCASE pops one CF entry per count and patches
\ all the ELSE jumps to point past the default body.
: CASE     ( -- )                 0 ; IMMEDIATE
: OF       ( -- )                 POSTPONE OVER POSTPONE = POSTPONE IF POSTPONE DROP ; IMMEDIATE
: ENDOF    ( count -- count+1 )   POSTPONE ELSE 1 + ; IMMEDIATE
: ENDCASE  ( count -- )           POSTPONE DROP BEGIN ?DUP WHILE 1 - POSTPONE THEN REPEAT ; IMMEDIATE

\ String demos: ." prints inline, S" pushes (addr count) for TYPE.
: HELLO     ( -- )  ." Hello, world!" CR ;
: GREETING  ( -- )  S" Hello from S-quote and TYPE." TYPE CR ;
