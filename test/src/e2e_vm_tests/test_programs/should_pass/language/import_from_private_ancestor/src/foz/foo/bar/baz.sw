library;

// This is legal even though foo is a private module, because foo is an ancestor of the current module
use ::foz::foo::*;
