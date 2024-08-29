script;

mod items_1;
mod lib_1;  // Item reexports of items_1
mod items_2_1;
mod items_2_2;
mod lib_2;  // Item reexports of items_2_1 and items_2_2
mod items_3_1;
mod lib_3;  // Item reexports of items_3_1 and items_3_2
mod items_4_1;
mod items_4_2;
mod items_4_3;
mod items_4_4;
mod lib_4;  // Item reexports of items_4_1 and items_4_2

mod tests; // All tests

fn main() -> u64 {
    tests::run_all_tests()
}
