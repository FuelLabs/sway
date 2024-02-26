script;

mod internal_mod;

use internal_mod::{*, internal_submod::*};
use external_mod::{*, external_submod::*};

// It should not be possible to import core contents via the standard library.
// It should not be possible to import standard library contents via submodules or external dependencies.

use std::core::ops::Eq;
use std::hash::core::ops::Add;
use internal_mod::internal_submod::std::hash::core::ops::Subtract;
use external_mod::external_submod::std::core::ops::Multiply;
use internal_mod::internal_submod::external_mod::external_submod::std::core::ops::Divide;

fn main() {
    assert(internal_mod_foo() +
	   internal_submod_foo() +
	   external_mod_foo() +
	   external_submod_foo() == 48);
}
