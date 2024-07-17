script;

mod items_1;
mod lib_1_1;  // Item reexports of items_1
mod lib_1_2;  // Item reexports of items_1

mod tests; // All tests

fn main() -> u64 {
    tests::run_all_tests()
}
