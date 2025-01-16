script;

pub mod items_1;
pub mod lib_1;  // Aliased item reexports of items_1
pub mod items_2;
pub mod lib_2;  // Aliased item reexports of items_2
pub mod items_3;
pub mod lib_3_1;  // Aliased item reexports of items_3
pub mod lib_3_2;  // Aliased item reexports of lib_3_1
pub mod items_4;
pub mod lib_4_1;  // Aliased item reexports of items_4
pub mod lib_4_2;  // Aliased item reexports of items_4 with different aliases

mod tests; // All tests

fn main() -> u64 {
    tests::run_all_tests()
}
