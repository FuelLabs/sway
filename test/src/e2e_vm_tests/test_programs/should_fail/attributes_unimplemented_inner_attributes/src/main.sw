//! This inner doc comment will not produce an error.
library;

// Only the unimplemented error for inner attributes must be emitted.

#![storage(invalid)]
#![unknown]
#![allow(dead_code, deprecated), unknown, storage(invalid)]
struct S { }