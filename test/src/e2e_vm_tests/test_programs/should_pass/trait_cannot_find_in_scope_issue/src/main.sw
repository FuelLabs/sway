script;

pub mod lib;
pub mod other_lib;
mod trait_impls;

// Previously this was a `should_fail` test that checked if we emitted an error
// related to the hash trait in sha256 trait constraint not being explicitly imported.

// After we changed trait constraints paths to be fully resolved, then lookup
// of those types now works transparently even without the trait being imported here.

// In fact the previous behavior was problematic since a locally defined trait with the
// same name would take precedence, thus making trait constraint type lookups in essen
// dynamically scoped.

use std::hash::sha256;

use ::lib::{S, A, function};
use ::trait_impls::*;

fn main() {
    let _ = sha256(123u8);

    let s = S {};
    s.method_01(0u8);
    s.method_02(A {});
    S::associated_function(A {});

    function(A {});

    let a = A {};

    a.trait_method(A {});

    A::trait_associated_function(A {});

    function_with_duplicated_trait(A {});
}
