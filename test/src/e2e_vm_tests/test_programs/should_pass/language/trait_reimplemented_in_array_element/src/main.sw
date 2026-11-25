//! TODO: Move this test to `should_fail` once
//!       https://github.com/FuelLabs/sway/issues/7759 is fixed.
//!
//! This program is invalid - it implements an already implemented trait.
//! However, it currently compiles, because the errors emitted in item
//! declarations inside of array element expressions are dropped:
//! https://github.com/FuelLabs/sway/issues/7759
//!
//! Until the above issue is fixed, this test pins the current behavior,
//! and, more importantly, tests that the decl slots left open by the
//! dropped failure are properly forgotten and do not cause an ICE.
library;

trait T {
    fn tf(self) -> u64;
}

struct S {}

pub fn main() -> u64 {
    let _a = [
        {
            impl T for S {
                fn tf(self) -> u64 {
                    1
                }
            }

            // Implementing an already implemented trait is an error,
            // but it is currently dropped here, together with the other
            // errors emitted in item declarations inside of array
            // element expressions.
            impl T for S {
                fn tf(self) -> u64 {
                    2
                }
            }

            1u64
        },
        2u64,
    ];

    42
}
