script;

mod items_1;
mod lib_1_1;  // Item reexports of items_1
mod lib_1_2;  // Item reexports of items_1
mod items_2;
mod lib_2_1;  // Star reexports of items_2
mod lib_2_2;  // Star reexports of items_2
mod items_3;
mod lib_3_1;  // Star reexports of items_3
mod lib_3_2;  // Item reexports of items_3
mod items_4;
mod lib_4_1;  // Item reexports of items_4
mod lib_4_2;  // Star reexports of items_4

mod tests; // All tests

fn main() -> u64 {
    tests::run_all_tests()
}
