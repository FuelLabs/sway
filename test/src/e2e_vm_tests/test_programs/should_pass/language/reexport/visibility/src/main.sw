script;

mod items_1;
mod lib_1_1;  // Item reexports of items_1
mod lib_1_2;  // Item imports without reexport of items_1
mod items_2;
mod lib_2_1;  // Item imports without reexport of items_1
mod lib_2_2;  // Item reexports of items_1

mod tests; // All tests

fn main() -> u64 {
    tests::run_all_tests()
}
