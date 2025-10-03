library;

pub mod items_1;
pub mod lib_1;  // Item reexports of items_1
pub mod items_2_1;
pub mod items_2_2;
pub mod lib_2;  // Item reexports of items_2_1 and items_2_2
pub mod items_3_1;
pub mod lib_3;  // Item reexports of items_3_1 and items_3_2
pub mod items_4_1;
pub mod items_4_2;
pub mod items_4_3;
pub mod items_4_4;
pub mod lib_4;  // Item reexports of items_4_1 and items_4_2

mod tests; // All tests

pub fn main() -> u64 {
    tests::run_all_tests()
}
