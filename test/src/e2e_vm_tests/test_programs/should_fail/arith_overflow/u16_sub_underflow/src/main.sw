script;

use core::codec::*;

fn main() -> bool {
    let a: u16 = u16::min();
    let b: u16 = 1;

    let result: u16 = a - b;
    log(result);

    true
}
