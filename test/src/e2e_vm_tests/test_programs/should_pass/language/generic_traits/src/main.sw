script;

// We should definitely implement something like the "fully qualified syntax",
// but until then, multiple methods with the same name is undefined behavior.
// https://doc.rust-lang.org/rust-by-example/trait/disambiguating.html

dep my_double;
dep my_point;
dep my_triple;
dep trait_tests;

use trait_tests::*;

fn main() -> u64 {
    run_test_traits();

    42
}
