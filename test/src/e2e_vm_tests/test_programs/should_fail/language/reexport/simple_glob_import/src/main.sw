library;

pub mod items_1;
pub mod lib_1;  // Item reexports of items_1
pub mod items_2;
pub mod lib_2;  // Item reexports of items_1

mod tests; // All tests

pub fn main() -> u64 {
    tests::run_all_tests()
}
