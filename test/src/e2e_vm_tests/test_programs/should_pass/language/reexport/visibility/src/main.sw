script;

pub mod items_1;
pub mod lib_1_1;  // Item reexports of items_1
pub mod lib_1_2;  // Item imports without reexport of items_1
pub mod items_2;
pub mod lib_2_1;  // Item imports without reexport of items_1
pub mod lib_2_2;  // Item reexports of items_1

mod tests; // All tests

fn main() -> u64 {
    tests::run_all_tests()
}
