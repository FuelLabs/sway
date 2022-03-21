script;
use std::*;

fn main() -> bool {
    let a: bool = true;
    let b = !a; // false
    let c = ! !b; // false
    let d = ! ! !c; // true
    d
}
