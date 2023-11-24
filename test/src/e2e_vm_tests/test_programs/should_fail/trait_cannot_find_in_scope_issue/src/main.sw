script;

mod lib;
mod other_lib;
mod trait_impls;

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
