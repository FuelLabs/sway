script;

enum X {
    Y: bool,
}

fn main() -> u64 {
    let x = X::Y(true);

    match x {
        X::Y(false) => { 24 },
        X::Y(true) => { 42 },
    }
}
