script;
use std::*;

fn main() -> bool {
    let a: bool = true;
    let b = !a; // false
    let c = !( !b); // false
    let d = !( ! !c); // true
    let e = ( ! ! !(d));
    !(and_true(a)) || true
}

fn and_true(x: bool) -> bool {
    let y = ! !x;
    x && true
}
