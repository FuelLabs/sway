script;

mod items_1;
mod lib_1;  // Aliased item reexports of items_1
mod items_2;
mod lib_2;  // Aliased item reexports of items_2
mod items_3;
mod lib_3;  // Aliased item reexports of items_3
mod items_4;
mod lib_4_1;  // Aliased item reexports of items_4
mod lib_4_2;  // Aliased item reexports of lib_4_1
mod items_5;
mod lib_5_1;  // Aliased trait reexports from items_5
mod lib_5_2;  // Aliased trait reexports from items_5

mod tests; // All tests

fn main() -> u64 {
    tests::run_all_tests()
}
