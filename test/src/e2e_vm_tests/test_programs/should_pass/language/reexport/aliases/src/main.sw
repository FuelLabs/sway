script;

mod items_1;
mod lib_1;  // Aliased item reexports of items_1
mod items_2;
mod lib_2;  // Aliased item reexports of items_2
mod items_3;
mod lib_3_1;  // Aliased item reexports of items_3
mod lib_3_2;  // Aliased item reexports of lib_3_1
mod items_4;
mod lib_4_1;  // Aliased item reexports of items_4
mod lib_4_2;  // Aliased item reexports of items_4 with different aliases

mod tests; // All tests

fn main() -> u64 {
    tests::run_all_tests()
}
