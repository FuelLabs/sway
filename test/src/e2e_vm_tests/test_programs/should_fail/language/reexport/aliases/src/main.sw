library;

pub mod items_1;
pub mod lib_1;  // Aliased item reexports of items_1
pub mod items_2;
pub mod lib_2;  // Aliased item reexports of items_2
pub mod items_3;
pub mod lib_3;  // Aliased item reexports of items_3
pub mod items_4;
pub mod lib_4_1;  // Aliased item reexports of items_4
pub mod lib_4_2;  // Aliased item reexports of lib_4_1
pub mod items_5;
pub mod lib_5_1;  // Aliased trait reexports from items_5
pub mod lib_5_2;  // Aliased trait reexports from items_5

mod tests; // All tests

pub fn main() -> u64 {
    tests::run_all_tests()
}
