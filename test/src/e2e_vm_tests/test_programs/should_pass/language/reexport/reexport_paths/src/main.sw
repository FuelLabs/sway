script;

pub mod items_1;
pub mod lib_1_1;  // Reexports of items_1
pub mod lib_1_2;  // Reexports of lib_1_1
pub mod lib_2;  // Reexports of std::hash::Hasher, which is not part of the std prelude
pub mod lib_3_1;  // Reexports of std::hash::Hash, which is not part of the std prelude
pub mod lib_3_2;  // Reexports of std::hash::Hash, which is not part of the std prelude
pub mod lib_4;  // Reexport of std::registers::global_gas
pub mod lib_5;  // Reexport of std::codec::*
//pub mod lib_6_1;  // Reexports of std::address::Address from the std prelude
pub mod lib_6_2;  // Reexports of std::address::Address directly from std::address

mod tests; // All tests

fn main() -> u64 {
    tests::run_all_tests()
}
