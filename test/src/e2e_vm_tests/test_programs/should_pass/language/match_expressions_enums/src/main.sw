script;

enum X {
    Y: u64,
}

fn main() -> u64 {
    let x = X::Y(42);

    match x {
        X::Y(hi) => { hi },
        _ => { 24 },
    }
}
