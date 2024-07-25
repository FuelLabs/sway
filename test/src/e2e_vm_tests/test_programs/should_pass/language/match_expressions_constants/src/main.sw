script;

mod lib;
mod top_level;
mod in_structs;

fn main() -> u64 {
    ::top_level::test();
    ::in_structs::test();
    42
}
