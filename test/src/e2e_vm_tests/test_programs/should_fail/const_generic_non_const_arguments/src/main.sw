script;

mod private_constants;

struct NotAConst {}

fn value<const N: u64>() -> u64 {
    N
}

fn shadowed_by_variable<const A: u64>() {
    let A = 7;
    let _ = value::<A>();
}

fn main() {
    shadowed_by_variable::<3>();
    let _ = value::<NotAConst>();
    let _ = value::<private_constants::A>();
}
