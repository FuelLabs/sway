script;

enum X {
    Y: u64,
    Z: bool
}

fn main() -> u64 {
    let a = X::Y(42);
    let b = match a {
        X::Y(hi) => { hi },
        X::Z(false) => { 0 },
        _ => { 0 },
    };
    
    b
}
