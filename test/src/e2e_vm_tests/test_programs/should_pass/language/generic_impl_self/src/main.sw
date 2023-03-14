script;

mod utils;
mod tests;

use tests::*;

fn main() -> u32 {
    generic_impl_self_test();
    result_impl_test();

    10u32
}
